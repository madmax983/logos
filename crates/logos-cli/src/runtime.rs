use std::collections::HashSet;
use std::env;
use std::fmt;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use blake3::Hasher;
use logos_core::{Correction, Posting, TransactionBuilder, TransactionId};
use logos_import::{
    CsvMapping, ImportError, ImportRecord, deterministic_fingerprint, parse_pdf_statement_file,
    parse_simple_csv_row,
};
use logos_reporting::{
    RegisterEntry, project_budget_variance, project_cashflow, project_register_balance,
};
use logos_store_aletheia::{
    AletheiaStore, StoreError,
    model::{
        NewImportRecord, StoredAnalyticsArtifactManifest, StoredReconciliationRun,
        StoredTransaction,
    },
};
use polars::prelude::{DataFrame, NamedFrom, ParquetWriter, Series};

const LOGOS_DB_PATH_ENV: &str = "LOGOS_DB_PATH";
const LOGOS_ARTIFACTS_PATH_ENV: &str = "LOGOS_ARTIFACTS_PATH";
const DEFAULT_DB_DIRECTORY: &str = ".logos";
const DEFAULT_DB_NAME: &str = "ledger";
const ARTIFACTS_DIRECTORY: &str = "artifacts";
const PARQUET_DIRECTORY: &str = "parquet";
const DEFAULT_ANALYTICS_SCHEMA_VERSION: i64 = 1;

#[derive(Debug)]
pub enum RuntimeError {
    Store(StoreError),
    Import(ImportError),
    Analytics { message: String },
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Store(err) => write!(f, "{err}"),
            Self::Import(err) => write!(f, "{err}"),
            Self::Analytics { message } => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for RuntimeError {}

impl From<StoreError> for RuntimeError {
    fn from(value: StoreError) -> Self {
        Self::Store(value)
    }
}

impl From<ImportError> for RuntimeError {
    fn from(value: ImportError) -> Self {
        Self::Import(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MonthReport {
    checking_balance: i64,
    income: i64,
    expense: i64,
    cashflow: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MonthReconciliation {
    ledger_delta_cents: i64,
    expected_closing_balance_cents: i64,
    statement_closing_balance_cents: i64,
    variance_cents: i64,
    reconciled: bool,
    matched_postings: usize,
    inflow_cents: i64,
    outflow_cents: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PdfImportSummary {
    imported_count: usize,
    duplicate_count: usize,
    dry_run: bool,
}

impl PdfImportSummary {
    #[must_use]
    pub const fn new(imported_count: usize, duplicate_count: usize, dry_run: bool) -> Self {
        Self {
            imported_count,
            duplicate_count,
            dry_run,
        }
    }

    #[must_use]
    pub const fn imported_count(&self) -> usize {
        self.imported_count
    }

    #[must_use]
    pub const fn duplicate_count(&self) -> usize {
        self.duplicate_count
    }

    #[must_use]
    pub const fn dry_run(&self) -> bool {
        self.dry_run
    }
}

impl MonthReport {
    #[must_use]
    pub const fn new(
        checking_balance_cents: i64,
        income_cents: i64,
        expense_cents: i64,
        cashflow_cents: i64,
    ) -> Self {
        Self {
            checking_balance: checking_balance_cents,
            income: income_cents,
            expense: expense_cents,
            cashflow: cashflow_cents,
        }
    }

    #[must_use]
    pub const fn checking_balance_cents(&self) -> i64 {
        self.checking_balance
    }

    #[must_use]
    pub const fn income_cents(&self) -> i64 {
        self.income
    }

    #[must_use]
    pub const fn expense_cents(&self) -> i64 {
        self.expense
    }

    #[must_use]
    pub const fn cashflow_cents(&self) -> i64 {
        self.cashflow
    }
}

impl MonthReconciliation {
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        ledger_delta_cents: i64,
        expected_closing_balance_cents: i64,
        statement_closing_balance_cents: i64,
        variance_cents: i64,
        reconciled: bool,
        matched_postings: usize,
        inflow_cents: i64,
        outflow_cents: i64,
    ) -> Self {
        Self {
            ledger_delta_cents,
            expected_closing_balance_cents,
            statement_closing_balance_cents,
            variance_cents,
            reconciled,
            matched_postings,
            inflow_cents,
            outflow_cents,
        }
    }

    #[must_use]
    pub const fn ledger_delta_cents(&self) -> i64 {
        self.ledger_delta_cents
    }

    #[must_use]
    pub const fn expected_closing_balance_cents(&self) -> i64 {
        self.expected_closing_balance_cents
    }

    #[must_use]
    pub const fn statement_closing_balance_cents(&self) -> i64 {
        self.statement_closing_balance_cents
    }

    #[must_use]
    pub const fn variance_cents(&self) -> i64 {
        self.variance_cents
    }

    #[must_use]
    pub const fn is_reconciled(&self) -> bool {
        self.reconciled
    }

    #[must_use]
    pub const fn matched_postings(&self) -> usize {
        self.matched_postings
    }

    #[must_use]
    pub const fn inflow_cents(&self) -> i64 {
        self.inflow_cents
    }

    #[must_use]
    pub const fn outflow_cents(&self) -> i64 {
        self.outflow_cents
    }
}

#[derive(Debug)]
pub struct CliRuntime {
    store: AletheiaStore,
    imported_records: usize,
    artifacts_root: PathBuf,
}

impl Default for CliRuntime {
    fn default() -> Self {
        let default_store_path = Self::default_store_path();
        Self {
            store: AletheiaStore::new_in_memory(),
            imported_records: 0,
            artifacts_root: default_artifacts_root(&default_store_path),
        }
    }
}

impl CliRuntime {
    /// Creates a runtime backed by the default durable store path.
    ///
    /// `LOGOS_DB_PATH` overrides the location. Otherwise, the path defaults to:
    /// - `${HOME}/.logos/ledger` on Unix-like systems
    /// - `%USERPROFILE%\\.logos\\ledger` on Windows
    /// - `./.logos/ledger` when no home directory is available
    ///
    /// # Errors
    ///
    /// Returns an error when opening the embedded store fails.
    pub fn new() -> Result<Self, RuntimeError> {
        Self::open(Self::default_store_path())
    }

    /// Creates a runtime pinned to a specific durable store path.
    ///
    /// # Errors
    ///
    /// Returns an error when opening the embedded store fails.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, RuntimeError> {
        let store_path = path.as_ref().to_path_buf();
        Ok(Self {
            store: AletheiaStore::open(&store_path)?,
            imported_records: 0,
            artifacts_root: default_artifacts_root(&store_path),
        })
    }

    #[must_use]
    pub fn new_in_memory() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn default_store_path() -> PathBuf {
        if let Some(path) = env::var_os(LOGOS_DB_PATH_ENV) {
            return PathBuf::from(path);
        }

        if let Some(home) = env::var_os("HOME").or_else(|| env::var_os("USERPROFILE")) {
            return PathBuf::from(home)
                .join(DEFAULT_DB_DIRECTORY)
                .join(DEFAULT_DB_NAME);
        }

        env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(DEFAULT_DB_DIRECTORY)
            .join(DEFAULT_DB_NAME)
    }

    /// Posts a balanced double-entry transaction and persists it.
    ///
    /// # Errors
    ///
    /// Returns an error when transaction validation or persistence fails.
    pub fn post_double_entry(
        &mut self,
        description: &str,
        debit_account: &str,
        credit_account: &str,
        amount_cents: i64,
    ) -> Result<TransactionId, RuntimeError> {
        let builder = TransactionBuilder::new(description)
            .posting(Posting::debit(debit_account, amount_cents))
            .posting(Posting::credit(credit_account, amount_cents));
        Ok(self.store.write_transaction(builder)?)
    }

    #[must_use]
    pub fn transaction_exists(&self, id: &TransactionId) -> bool {
        self.store.has_transaction(id)
    }

    #[must_use]
    pub fn register_balance_for(&self, account: &str) -> i64 {
        let entries: Vec<RegisterEntry> = self
            .store
            .transactions()
            .flat_map(|stored| stored.transaction().postings().iter())
            .filter(|posting| posting.account() == account)
            .map(|posting| RegisterEntry::new(posting.amount()))
            .collect();

        project_register_balance(0, &entries)
    }

    #[must_use]
    pub fn budget_variance_for(&self, budget_cents: i64, expense_account_prefix: &str) -> i64 {
        let actual_expense_cents: i64 = self
            .store
            .transactions()
            .flat_map(|stored| stored.transaction().postings().iter())
            .filter(|posting| posting.account().starts_with(expense_account_prefix))
            .map(Posting::amount)
            .filter(|amount| *amount > 0)
            .sum();
        project_budget_variance(budget_cents, actual_expense_cents)
    }

    /// Persists a budget target for a month and account-prefix scope.
    ///
    /// # Errors
    ///
    /// Returns an error when persistence fails.
    pub fn set_budget_target_for_month(
        &mut self,
        month_key: &str,
        expense_account_prefix: &str,
        budget_cents: i64,
    ) -> Result<(), RuntimeError> {
        self.store
            .write_budget_target(month_key, expense_account_prefix, budget_cents)?;
        Ok(())
    }

    #[must_use]
    pub fn budget_target_for_month(
        &self,
        month_key: &str,
        expense_account_prefix: &str,
    ) -> Option<i64> {
        self.store
            .budget_target(month_key, expense_account_prefix)
            .map(logos_store_aletheia::model::StoredBudgetTarget::budget_cents)
    }

    #[must_use]
    pub fn budget_variance_for_month(
        &self,
        month_key: &str,
        budget_cents: i64,
        expense_account_prefix: &str,
    ) -> i64 {
        let actual_expense_cents = self.expense_total_for_month(month_key, expense_account_prefix);
        project_budget_variance(budget_cents, actual_expense_cents)
    }

    #[must_use]
    pub fn month_report_for(&self, checking_account: &str, month_key: &str) -> MonthReport {
        let checking_balance_cents: i64 = self
            .store
            .transactions()
            .filter(|stored| transaction_in_month(stored, month_key))
            .flat_map(|stored| stored.transaction().postings().iter())
            .filter(|posting| posting.account() == checking_account)
            .map(Posting::amount)
            .sum();

        let income_cents: i64 = self
            .store
            .transactions()
            .filter(|stored| transaction_in_month(stored, month_key))
            .flat_map(|stored| stored.transaction().postings().iter())
            .filter(|posting| posting.account().starts_with("income:"))
            .map(Posting::amount)
            .filter(|amount| *amount < 0)
            .map(i64::abs)
            .sum();

        let expense_cents: i64 = self
            .store
            .transactions()
            .filter(|stored| transaction_in_month(stored, month_key))
            .flat_map(|stored| stored.transaction().postings().iter())
            .filter(|posting| posting.account().starts_with("expenses:"))
            .map(Posting::amount)
            .filter(|amount| *amount > 0)
            .sum();

        let cashflow_cents = project_cashflow(income_cents, expense_cents);
        MonthReport::new(
            checking_balance_cents,
            income_cents,
            expense_cents,
            cashflow_cents,
        )
    }

    #[must_use]
    pub fn reconcile_month_for(
        &self,
        checking_account: &str,
        month_key: &str,
        opening_balance_cents: i64,
        closing_balance_cents: i64,
    ) -> MonthReconciliation {
        let mut ledger_delta_cents = 0_i64;
        let mut inflow_cents = 0_i64;
        let mut outflow_cents = 0_i64;
        let mut matched_postings = 0_usize;

        for posting in self
            .store
            .transactions()
            .filter(|stored| transaction_in_month(stored, month_key))
            .flat_map(|stored| stored.transaction().postings().iter())
            .filter(|posting| posting.account() == checking_account)
        {
            let amount = posting.amount();
            matched_postings = matched_postings.saturating_add(1);
            ledger_delta_cents = ledger_delta_cents.saturating_add(amount);
            if amount >= 0 {
                inflow_cents = inflow_cents.saturating_add(amount);
            } else {
                outflow_cents =
                    outflow_cents.saturating_add(amount.checked_abs().unwrap_or(i64::MAX));
            }
        }

        let expected_closing_balance_cents =
            opening_balance_cents.saturating_add(ledger_delta_cents);
        let variance_cents = closing_balance_cents.saturating_sub(expected_closing_balance_cents);
        MonthReconciliation::new(
            ledger_delta_cents,
            expected_closing_balance_cents,
            closing_balance_cents,
            variance_cents,
            variance_cents == 0,
            matched_postings,
            inflow_cents,
            outflow_cents,
        )
    }

    /// Computes and persists an immutable reconciliation run for the provided month and account.
    ///
    /// # Errors
    ///
    /// Returns an error when reconciliation persistence fails.
    pub fn reconcile_and_persist_month_for(
        &mut self,
        checking_account: &str,
        month_key: &str,
        opening_balance_cents: i64,
        closing_balance_cents: i64,
    ) -> Result<StoredReconciliationRun, RuntimeError> {
        let report = self.reconcile_month_for(
            checking_account,
            month_key,
            opening_balance_cents,
            closing_balance_cents,
        );
        let reconciled_txn_ids =
            self.reconciliation_transaction_ids_for(checking_account, month_key);
        let matched_postings = i64::try_from(report.matched_postings()).unwrap_or(i64::MAX);
        self.store
            .write_reconciliation_run(
                month_key,
                checking_account,
                opening_balance_cents,
                report.ledger_delta_cents(),
                report.expected_closing_balance_cents(),
                closing_balance_cents,
                report.variance_cents(),
                report.is_reconciled(),
                matched_postings,
                report.inflow_cents(),
                report.outflow_cents(),
                &reconciled_txn_ids,
            )
            .map_err(RuntimeError::from)
    }

    #[must_use]
    pub fn reconciliation_run_count(&self) -> usize {
        self.store.reconciliation_run_count()
    }

    #[must_use]
    pub fn reconciliation_run(&self, run_id: &str) -> Option<StoredReconciliationRun> {
        self.store.reconciliation_run(run_id).cloned()
    }

    #[must_use]
    pub fn list_reconciliation_runs(
        &self,
        month_key: Option<&str>,
        checking_account: Option<&str>,
    ) -> Vec<StoredReconciliationRun> {
        let mut runs: Vec<_> = self
            .store
            .reconciliation_runs()
            .filter(|run| month_key.is_none_or(|month| run.month_key() == month))
            .filter(|run| checking_account.is_none_or(|account| run.checking_account() == account))
            .cloned()
            .collect();
        runs.sort_by(|left, right| {
            right
                .created_at()
                .wallclock()
                .cmp(&left.created_at().wallclock())
                .then_with(|| left.run_id().cmp(right.run_id()))
        });
        runs
    }

    #[must_use]
    pub fn current_month_key_utc() -> String {
        let wallclock_us = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_micros())
            .ok()
            .and_then(|micros| i64::try_from(micros).ok())
            .unwrap_or(0);
        month_key_from_wallclock_utc(wallclock_us)
    }

    /// Imports one CSV row with deterministic idempotency.
    ///
    /// Returns `true` when newly inserted, `false` when duplicate.
    ///
    /// # Errors
    ///
    /// Returns an error when parsing fails.
    pub fn import_csv_row(
        &mut self,
        row: &str,
        mapping: &CsvMapping,
    ) -> Result<bool, RuntimeError> {
        let record = parse_simple_csv_row(row, mapping)?;
        let content_hash_key = import_content_hash_key(deterministic_fingerprint(&record));
        if self.store.has_import_record_content_hash(&content_hash_key) {
            return Ok(false);
        }

        let batch_key = import_batch_key(
            "csv-row",
            "inline:csv",
            false,
            false,
            std::slice::from_ref(&content_hash_key),
        );
        self.store.write_import_batch(
            "csv-row",
            "inline:csv",
            &batch_key,
            0,
            false,
            false,
            &[NewImportRecord::new(&content_hash_key, None)],
        )?;
        self.imported_records = self.imported_records.saturating_add(1);
        Ok(true)
    }

    /// Imports transaction rows from a PDF statement.
    ///
    /// When `dry_run` is true, records are parsed and deduplicated but not posted.
    ///
    /// # Errors
    ///
    /// Returns an error when parsing fails or posting a derived transaction fails.
    pub fn import_pdf_statement(
        &mut self,
        path: impl AsRef<Path>,
        account: &str,
        dry_run: bool,
        enable_ocr: bool,
    ) -> Result<PdfImportSummary, RuntimeError> {
        let source_uri = path.as_ref().display().to_string();
        let records = parse_pdf_statement_file(&path, account, enable_ocr)?;
        let mut imported_count = 0_usize;
        let mut duplicate_count = 0_usize;
        let mut seen_in_call: HashSet<String> = HashSet::new();
        let mut imported_records = Vec::new();
        let mut imported_keys = Vec::new();

        for record in records {
            let content_hash_key = import_content_hash_key(deterministic_fingerprint(&record));
            let seen_previously = self.store.has_import_record_content_hash(&content_hash_key);
            let seen_in_batch = !seen_in_call.insert(content_hash_key.clone());
            if seen_previously || seen_in_batch {
                duplicate_count = duplicate_count.saturating_add(1);
                continue;
            }

            imported_count = imported_count.saturating_add(1);
            if dry_run {
                continue;
            }

            let txn_id = self.post_import_record(&record)?;
            imported_records.push(NewImportRecord::new(&content_hash_key, Some(&txn_id)));
            imported_keys.push(content_hash_key);
            self.imported_records = self.imported_records.saturating_add(1);
        }

        if !dry_run {
            let batch_key = import_batch_key(
                "pdf-statement",
                &source_uri,
                dry_run,
                enable_ocr,
                &imported_keys,
            );
            let duplicate_count = i64::try_from(duplicate_count).unwrap_or(i64::MAX);
            self.store.write_import_batch(
                "pdf-statement",
                &source_uri,
                &batch_key,
                duplicate_count,
                dry_run,
                enable_ocr,
                &imported_records,
            )?;
        }

        Ok(PdfImportSummary::new(
            imported_count,
            duplicate_count,
            dry_run,
        ))
    }

    #[must_use]
    pub const fn imported_record_count(&self) -> usize {
        self.imported_records
    }

    #[must_use]
    pub const fn default_analytics_schema_version() -> i64 {
        DEFAULT_ANALYTICS_SCHEMA_VERSION
    }

    /// Creates an immutable Parquet analytics snapshot and records its manifest in Aletheia.
    ///
    /// # Errors
    ///
    /// Returns an error when snapshot reconstruction, parquet writing, or manifest persistence fails.
    pub fn create_analytics_snapshot(
        &mut self,
        as_of_valid_time_us: Option<i64>,
        as_of_tx_time_us: Option<i64>,
        schema_version: i64,
        supersedes_artifact_id: Option<&str>,
    ) -> Result<StoredAnalyticsArtifactManifest, RuntimeError> {
        if schema_version <= 0 {
            return Err(RuntimeError::Analytics {
                message: format!("schema_version must be positive, got {schema_version}"),
            });
        }

        let snapshot_now = current_time_us();
        let as_of_valid = as_of_valid_time_us.unwrap_or(snapshot_now);
        let as_of_tx = as_of_tx_time_us.unwrap_or(snapshot_now);
        let rows = snapshot_rows(self.store.transactions_as_of_us(as_of_valid, as_of_tx)?);
        let content_hash = hash_rows(&rows, as_of_valid, as_of_tx, schema_version);

        let parquet_dir = self.artifacts_root.join(PARQUET_DIRECTORY);
        fs::create_dir_all(&parquet_dir).map_err(|err| RuntimeError::Analytics {
            message: format!(
                "failed to create parquet artifact directory '{}': {err}",
                parquet_dir.display()
            ),
        })?;

        let parquet_path = parquet_dir.join(format!("{content_hash}.parquet"));
        if !parquet_path.exists() {
            write_rows_to_parquet(&parquet_path, &rows, as_of_valid, as_of_tx)?;
        }

        let parquet_uri = parquet_path.display().to_string();
        self.store
            .write_analytics_artifact_manifest_us(
                "parquet",
                &parquet_uri,
                &content_hash,
                schema_version,
                i64::try_from(rows.len()).unwrap_or(i64::MAX),
                as_of_valid,
                as_of_tx,
                supersedes_artifact_id,
            )
            .map_err(RuntimeError::from)
    }

    #[must_use]
    pub fn list_analytics_snapshots(&self) -> Vec<StoredAnalyticsArtifactManifest> {
        let mut manifests: Vec<_> = self.store.analytics_artifacts().cloned().collect();
        manifests.sort_by(|left, right| {
            right
                .created_at()
                .wallclock()
                .cmp(&left.created_at().wallclock())
                .then_with(|| left.artifact_id().cmp(right.artifact_id()))
        });
        manifests
    }

    #[must_use]
    pub fn get_analytics_snapshot(
        &self,
        artifact_id: &str,
    ) -> Option<StoredAnalyticsArtifactManifest> {
        self.store.analytics_artifact(artifact_id).cloned()
    }

    /// Applies an append-only correction to a previously written transaction.
    ///
    /// # Errors
    ///
    /// Returns an error when correction creation or persistence fails.
    pub fn apply_correction(
        &mut self,
        supersedes_id: TransactionId,
        reason: &str,
    ) -> Result<(), RuntimeError> {
        let correction = Correction::new(supersedes_id, reason).map_err(StoreError::from)?;
        self.store.write_correction(correction)?;
        Ok(())
    }

    #[must_use]
    pub fn latest_correction_target(&self) -> Option<TransactionId> {
        self.store
            .latest_correction()
            .map(|correction| correction.supersedes_id().clone())
    }

    fn post_import_record(&mut self, record: &ImportRecord) -> Result<TransactionId, RuntimeError> {
        let amount = record.amount_cents();
        if amount > 0 {
            return self.post_double_entry(
                record.memo(),
                record.account(),
                record.category(),
                amount,
            );
        }

        let debit_amount = amount.checked_abs().ok_or(ImportError::InvalidAmount)?;
        self.post_double_entry(
            record.memo(),
            record.category(),
            record.account(),
            debit_amount,
        )
    }

    fn expense_total_for_month(&self, month_key: &str, expense_account_prefix: &str) -> i64 {
        self.store
            .transactions()
            .filter(|stored| transaction_in_month(stored, month_key))
            .flat_map(|stored| stored.transaction().postings().iter())
            .filter(|posting| posting.account().starts_with(expense_account_prefix))
            .map(Posting::amount)
            .filter(|amount| *amount > 0)
            .sum()
    }

    fn reconciliation_transaction_ids_for(
        &self,
        checking_account: &str,
        month_key: &str,
    ) -> Vec<TransactionId> {
        let mut ids: Vec<_> = self
            .store
            .transactions()
            .filter(|stored| transaction_in_month(stored, month_key))
            .filter(|stored| {
                stored
                    .transaction()
                    .postings()
                    .iter()
                    .any(|posting| posting.account() == checking_account)
            })
            .map(|stored| stored.id().clone())
            .collect();
        ids.sort_by(|left, right| left.as_str().cmp(right.as_str()));
        ids.dedup_by(|left, right| left.as_str() == right.as_str());
        ids
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SnapshotPostingRow {
    txn_id: String,
    description: String,
    effective_at_us: i64,
    posting_ordinal: i64,
    account: String,
    amount_cents: i64,
}

fn import_content_hash_key(fingerprint: u64) -> String {
    format!("{fingerprint:016x}")
}

fn import_batch_key(
    import_kind: &str,
    source_uri: &str,
    dry_run: bool,
    ocr_enabled: bool,
    imported_keys: &[String],
) -> String {
    let mut sorted_keys = imported_keys.to_vec();
    sorted_keys.sort();

    let mut hasher = Hasher::new();
    hasher.update(
        format!("kind:{import_kind}|source:{source_uri}|dry_run:{dry_run}|ocr:{ocr_enabled}\n")
            .as_bytes(),
    );
    for key in sorted_keys {
        hasher.update(key.as_bytes());
        hasher.update(b"\n");
    }
    hasher.finalize().to_hex().to_string()
}

fn default_artifacts_root(store_path: &Path) -> PathBuf {
    if let Some(path) = env::var_os(LOGOS_ARTIFACTS_PATH_ENV) {
        return PathBuf::from(path);
    }

    store_path.parent().map_or_else(
        || store_path.join(ARTIFACTS_DIRECTORY),
        |parent| parent.join(ARTIFACTS_DIRECTORY),
    )
}

fn current_time_us() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_micros())
        .ok()
        .and_then(|micros| i64::try_from(micros).ok())
        .unwrap_or(0)
}

fn snapshot_rows(transactions: Vec<StoredTransaction>) -> Vec<SnapshotPostingRow> {
    let mut rows = Vec::new();
    for stored in transactions {
        for (index, posting) in stored.transaction().postings().iter().enumerate() {
            let posting_ordinal = i64::try_from(index).unwrap_or(i64::MAX);
            rows.push(SnapshotPostingRow {
                txn_id: stored.id().as_str().to_owned(),
                description: stored.transaction().description().to_owned(),
                effective_at_us: stored.effective_at().wallclock(),
                posting_ordinal,
                account: posting.account().to_owned(),
                amount_cents: posting.amount(),
            });
        }
    }

    rows.sort_by(|left, right| {
        left.txn_id
            .cmp(&right.txn_id)
            .then_with(|| left.posting_ordinal.cmp(&right.posting_ordinal))
            .then_with(|| left.account.cmp(&right.account))
            .then_with(|| left.amount_cents.cmp(&right.amount_cents))
    });
    rows
}

fn hash_rows(
    rows: &[SnapshotPostingRow],
    as_of_valid: i64,
    as_of_tx: i64,
    schema_version: i64,
) -> String {
    let mut hasher = Hasher::new();
    hasher
        .update(format!("schema:{schema_version}|valid:{as_of_valid}|tx:{as_of_tx}\n").as_bytes());
    for row in rows {
        hasher.update(
            format!(
                "{}|{}|{}|{}|{}|{}\n",
                row.txn_id,
                row.description,
                row.effective_at_us,
                row.posting_ordinal,
                row.account,
                row.amount_cents
            )
            .as_bytes(),
        );
    }
    hasher.finalize().to_hex().to_string()
}

fn write_rows_to_parquet(
    parquet_path: &Path,
    rows: &[SnapshotPostingRow],
    as_of_valid: i64,
    as_of_tx: i64,
) -> Result<(), RuntimeError> {
    let txn_ids: Vec<String> = rows.iter().map(|row| row.txn_id.clone()).collect();
    let descriptions: Vec<String> = rows.iter().map(|row| row.description.clone()).collect();
    let effective_at_values: Vec<i64> = rows.iter().map(|row| row.effective_at_us).collect();
    let posting_ordinals: Vec<i64> = rows.iter().map(|row| row.posting_ordinal).collect();
    let accounts: Vec<String> = rows.iter().map(|row| row.account.clone()).collect();
    let amounts: Vec<i64> = rows.iter().map(|row| row.amount_cents).collect();
    let snapshot_valid_values = vec![as_of_valid; rows.len()];
    let snapshot_tx_values = vec![as_of_tx; rows.len()];

    let mut frame = DataFrame::new(vec![
        Series::new("txn_id".into(), txn_ids).into(),
        Series::new("description".into(), descriptions).into(),
        Series::new("effective_at_us".into(), effective_at_values).into(),
        Series::new("posting_ordinal".into(), posting_ordinals).into(),
        Series::new("account".into(), accounts).into(),
        Series::new("amount_cents".into(), amounts).into(),
        Series::new("snapshot_valid_at_us".into(), snapshot_valid_values).into(),
        Series::new("snapshot_tx_at_us".into(), snapshot_tx_values).into(),
    ])
    .map_err(|err| RuntimeError::Analytics {
        message: format!("failed to construct analytics frame: {err}"),
    })?;

    let file = File::create(parquet_path).map_err(|err| RuntimeError::Analytics {
        message: format!(
            "failed to create parquet snapshot '{}': {err}",
            parquet_path.display()
        ),
    })?;
    ParquetWriter::new(file)
        .finish(&mut frame)
        .map_err(|err| RuntimeError::Analytics {
            message: format!(
                "failed to write parquet snapshot '{}': {err}",
                parquet_path.display()
            ),
        })?;
    Ok(())
}

fn transaction_in_month(stored: &StoredTransaction, month_key: &str) -> bool {
    month_key_from_wallclock_utc(stored.effective_at().wallclock()) == month_key
}

fn month_key_from_wallclock_utc(wallclock_us: i64) -> String {
    let secs = wallclock_us.div_euclid(1_000_000);
    let days = secs.div_euclid(86_400);
    let (year, month, _) = civil_from_days(days);
    format!("{year:04}-{month:02}")
}

fn civil_from_days(days_since_unix_epoch: i64) -> (i64, u32, u32) {
    let z = days_since_unix_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let day_of_era = z - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let mut year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    if month <= 2 {
        year += 1;
    }
    let month_u32 = u32::try_from(month).unwrap_or(1);
    let day_u32 = u32::try_from(day).unwrap_or(1);
    (year, month_u32, day_u32)
}
