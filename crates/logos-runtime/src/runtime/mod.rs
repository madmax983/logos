struct SnapshotPostingRow {
    txn_id: String,
    description: String,
    effective_at_us: i64,
    posting_ordinal: i64,
    account: String,
    amount_cents: i64,
}

struct ResolvedAutopilotBalances {
    opening_balance_cents: i64,
    closing_balance_cents: i64,
}

use blake3::Hasher;
use chrono::{DateTime, Local, NaiveDate, NaiveDateTime, Utc};
use logos_core::{Correction, Posting, TransactionBuilder, TransactionId};
use logos_fetch::{
    FakeStatementAdapter, FetchRequest, FetchRunStatus, FetchedStatementArtifact,
    OnePasswordCliSecretResolver, OutputFormat, ProvidentAdapter, SecretBundle, SecretResolver,
    StatementAdapter, StatementSource, StatementSourceConfig,
};
use logos_import::{
    CsvMapping, ImportError, ImportRecord, deterministic_fingerprint,
    deterministic_fingerprint_legacy_v1, parse_pdf_statement_file, parse_simple_csv_row,
};
use logos_reporting::{
    RegisterEntry, RsuBudgetPlan, RsuBudgetPlanInput, ScenarioPriceInputs, project_budget_variance,
    project_cashflow, project_register_balance_iter, project_rsu_budget_plan,
};
use logos_store::{
    MemoryStore,
    error::StoreError,
    model::{
        NewImportRecord, StoredAnalyticsArtifactManifest, StoredFetchArtifactFormat,
        StoredFetchRun, StoredFetchRunStatus, StoredMonthClose, StoredReconciliationRun,
        StoredStatementLine, StoredTransaction,
    },
    traits::LedgerStore,
};
use polars::prelude::{DataFrame, ParquetWriter, Series};
use std::collections::HashSet;
use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::RuntimeError;
use crate::models::{
    CsvImportSummary, MonthAutopilotRequest, MonthAutopilotSummary, MonthReconciliation,
    MonthReport, PdfImportSummary,
};

const LOGOS_ARTIFACTS_PATH_ENV: &str = "LOGOS_ARTIFACTS_PATH";
const LOGOS_FETCH_CONFIG_PATH_ENV: &str = "LOGOS_FETCH_CONFIG_PATH";
const DEFAULT_STATE_DIRECTORY: &str = ".logos";
const DEFAULT_FETCH_CONFIG_NAME: &str = "statement-sources.toml";
const ARTIFACTS_DIRECTORY: &str = "artifacts";
const PARQUET_DIRECTORY: &str = "parquet";
const DEFAULT_ANALYTICS_SCHEMA_VERSION: i64 = 1;

#[derive(Debug)]
pub struct AppRuntime<S> {
    store: S,
    imported_records: usize,
    artifacts_root: PathBuf,
    fetch_config_path: Option<PathBuf>,
    fetched_statement_artifacts: Vec<FetchedStatementArtifact>,
}

impl<S> AppRuntime<S> {
    #[must_use]
    pub const fn with_store(
        store: S,
        artifacts_root: PathBuf,
        fetch_config_path: Option<PathBuf>,
    ) -> Self {
        Self {
            store,
            imported_records: 0,
            artifacts_root,
            fetch_config_path,
            fetched_statement_artifacts: Vec::new(),
        }
    }
}

impl AppRuntime<MemoryStore> {
    #[must_use]
    pub fn new_in_memory() -> Self {
        let state_root = default_state_root();
        Self::with_store(
            MemoryStore::new_in_memory(),
            default_artifacts_root(&state_root),
            Some(default_fetch_config_path(&state_root)),
        )
    }
}

impl<S> AppRuntime<S> {
    #[must_use]
    pub fn current_month_key_local() -> String {
        Local::now().format("%Y-%m").to_string()
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
    #[must_use]
    pub const fn default_analytics_schema_version() -> i64 {
        DEFAULT_ANALYTICS_SCHEMA_VERSION
    }
    fn secret_bundle_for_fetch_source(
        source: &StatementSource,
    ) -> Result<SecretBundle, RuntimeError> {
        let resolver = OnePasswordCliSecretResolver::from_environment();
        Self::secret_bundle_for_fetch_source_with_resolver(source, &resolver)
    }
    fn secret_bundle_for_fetch_source_with_resolver<R>(
        source: &StatementSource,
        resolver: &R,
    ) -> Result<SecretBundle, RuntimeError>
    where
        R: SecretResolver,
    {
        match source.institution_id() {
            "fake-fixture" => SecretBundle::new("fixture-user", "fixture-pass", Some("000000"))
                .map_err(|err| fetch_error_to_runtime(&err)),
            "fake-needs-attention" => {
                SecretBundle::new("fixture-user", "fixture-pass", Some("000000"))
                    .map_err(|err| fetch_error_to_runtime(&err))
            }
            _ => resolver
                .resolve(source)
                .map_err(|err| fetch_error_to_runtime(&err)),
        }
    }
}

impl<S: LedgerStore> AppRuntime<S> {
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
        let builder = build_double_entry(description, debit_account, credit_account, amount_cents)?;
        Ok(self.store.write_transaction(builder)?)
    }

    #[must_use]
    pub fn transaction_exists(&self, id: &TransactionId) -> bool {
        self.store.has_transaction(id)
    }

    /// Returns transactions visible at a bi-temporal as-of point (valid-time, tx-time).
    ///
    /// # Errors
    ///
    /// Returns an error when underlying as-of query execution fails.
    pub fn transactions_as_of_us(
        &self,
        as_of_valid_time_us: i64,
        as_of_tx_time_us: i64,
    ) -> Result<Vec<StoredTransaction>, RuntimeError> {
        self.store
            .transactions_as_of_us(as_of_valid_time_us, as_of_tx_time_us)
            .map_err(RuntimeError::from)
    }

    #[must_use]
    pub fn register_balance_for(&self, account: &str) -> i64 {
        let transactions = self.store.transactions();
        let entries = transactions
            .iter()
            .flat_map(|stored| stored.transaction().postings())
            .filter(|posting| posting.account().as_str() == account)
            .map(|posting| RegisterEntry::new(posting.amount()));

        project_register_balance_iter(0, entries)
    }

    #[must_use]
    pub fn budget_variance_for(&self, budget_cents: i64, expense_account_prefix: &str) -> i64 {
        let mut actual_expense_cents = 0_i64;
        for stored in self.store.transactions() {
            for posting in stored.transaction().postings() {
                if posting
                    .account()
                    .as_str()
                    .starts_with(expense_account_prefix)
                {
                    let amount = posting.amount();
                    if amount > 0 {
                        actual_expense_cents = actual_expense_cents.saturating_add(amount);
                    }
                }
            }
        }

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
            .map(|target| target.budget_cents())
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

    /// Computes the monthly report.
    ///
    /// ⚡ Bolt Optimization: Aggregates `checking_balance_cents`, `income_cents`, and `expense_cents`
    /// in a single pass over the transactions iterator, preventing multiple redundant iterations
    /// over the entire ledger history and using `saturating_add` to prevent overflow panics.
    #[must_use]
    pub fn month_report_for(&self, checking_account: &str, month_key: &str) -> MonthReport {
        let mut checking_balance_cents = 0_i64;
        let mut income_cents = 0_i64;
        let mut expense_cents = 0_i64;

        for stored in self
            .store
            .transactions()
            .into_iter()
            .filter(|stored| transaction_in_month(stored, month_key))
        {
            for posting in stored.transaction().postings() {
                let account = posting.account().as_str();
                let amount = posting.amount();

                if account == checking_account {
                    checking_balance_cents = checking_balance_cents.saturating_add(amount);
                }

                if account.starts_with("income:") && amount < 0 {
                    income_cents =
                        income_cents.saturating_add(amount.checked_abs().unwrap_or(i64::MAX));
                } else if account.starts_with("expenses:") && amount > 0 {
                    expense_cents = expense_cents.saturating_add(amount);
                }
            }
        }

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

        for stored in self
            .store
            .transactions()
            .into_iter()
            .filter(|stored| transaction_in_month(stored, month_key))
        {
            for posting in stored
                .transaction()
                .postings()
                .iter()
                .filter(|posting| posting.account().as_str() == checking_account)
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
        self.store.reconciliation_run(run_id)
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
            .into_iter()
            .filter(|run| month_key.is_none_or(|month| run.month_key() == month))
            .filter(|run| checking_account.is_none_or(|account| run.checking_account() == account))
            .collect();
        runs.sort_by(|left, right| {
            right
                .created_at()
                .cmp(&left.created_at())
                .then_with(|| left.run_id().cmp(right.run_id()))
        });
        runs
    }

    #[must_use]
    pub fn fetch_run(&self, run_id: &str) -> Option<StoredFetchRun> {
        self.store.fetch_run(run_id)
    }

    #[must_use]
    pub fn list_fetch_runs(
        &self,
        month_key: Option<&str>,
        checking_account: Option<&str>,
    ) -> Vec<StoredFetchRun> {
        let mut runs: Vec<_> = self
            .store
            .fetch_runs()
            .into_iter()
            .filter(|run| month_key.is_none_or(|month| run.month_key() == month))
            .filter(|run| checking_account.is_none_or(|account| run.ledger_account() == account))
            .collect();
        runs.sort_by(|left, right| {
            right
                .created_at()
                .cmp(&left.created_at())
                .then_with(|| left.run_id().cmp(right.run_id()))
        });
        runs
    }

    /// Closes a month scope using a previously persisted reconciliation run.
    ///
    /// # Errors
    ///
    /// Returns an error when scope metadata is invalid or write persistence fails.
    pub fn close_month(
        &mut self,
        month_key: &str,
        checking_account: &str,
        reconciliation_run_id: &str,
        analytics_artifact_id: Option<&str>,
    ) -> Result<StoredMonthClose, RuntimeError> {
        self.store
            .write_month_close(
                month_key,
                checking_account,
                reconciliation_run_id,
                analytics_artifact_id,
            )
            .map_err(RuntimeError::from)
    }

    #[must_use]
    pub fn month_close_for_scope(
        &self,
        month_key: &str,
        checking_account: &str,
    ) -> Option<StoredMonthClose> {
        self.store
            .month_close_for_scope(month_key, checking_account)
    }

    /// Stages fetched statement metadata for later month-autopilot resolution.
    pub fn stage_fetched_statement_artifact(&mut self, artifact: FetchedStatementArtifact) {
        self.fetched_statement_artifacts.retain(|existing| {
            !(existing.source_id() == artifact.source_id()
                && existing.ledger_account() == artifact.ledger_account()
                && existing.month_key() == artifact.month_key())
        });
        self.fetched_statement_artifacts.push(artifact);
    }

    /// Runs the full month workflow with close-time safety checks.
    ///
    /// Workflow:
    /// 1. optional statement PDF import
    /// 2. reconciliation preview + variance gate
    /// 3. reconciliation persistence
    /// 4. month report projection
    /// 5. immutable month close
    ///
    /// # Errors
    ///
    /// Returns an error when confirmation is missing, scope is already closed,
    /// variance safety checks fail, or any underlying import/store operation fails.
    pub fn run_month_autopilot(
        &mut self,
        request: &MonthAutopilotRequest,
    ) -> Result<MonthAutopilotSummary, RuntimeError> {
        if !request.confirm_close() {
            return Err(RuntimeError::Analytics {
                message: "month autopilot requires --confirm-close to persist month close"
                    .to_owned(),
            });
        }
        if self
            .month_close_for_scope(request.month_key(), request.checking_account())
            .is_some()
        {
            return Err(RuntimeError::Analytics {
                message: format!(
                    "month scope '{}' for '{}' is already closed",
                    request.month_key(),
                    request.checking_account()
                ),
            });
        }

        let needs_fetched_balances = !matches!(
            (
                request.opening_balance_cents(),
                request.closing_balance_cents(),
            ),
            (Some(_), Some(_))
        );
        let fetch_runs = if needs_fetched_balances
            && request.statement_pdf_path().is_none()
            && self
                .fetched_statement_artifact_for(request.checking_account(), request.month_key())
                .is_none()
        {
            self.fetch_configured_statement_artifacts(request, needs_fetched_balances)?
        } else {
            Vec::new()
        };

        let (imported_count, duplicate_count) = if let Some(path) = request.statement_pdf_path() {
            let summary = self.import_pdf_statement(
                path,
                request.checking_account(),
                false,
                request.enable_ocr(),
            )?;
            (summary.imported_count(), summary.duplicate_count())
        } else if let Some(artifact) =
            self.fetched_statement_artifact_for(request.checking_account(), request.month_key())
        {
            self.import_fetched_statement_artifact(&artifact, request.enable_ocr())?
        } else {
            (0, 0)
        };

        let balances = self.resolve_autopilot_balances(request)?;
        let preview = self.reconcile_month_for(
            request.checking_account(),
            request.month_key(),
            balances.opening_balance_cents,
            balances.closing_balance_cents,
        );
        if preview.variance_cents() != 0 && !request.allow_variance() {
            return Err(RuntimeError::Analytics {
                message: format!(
                    "month autopilot blocked close due to variance_cents={} (use --allow-variance to override)",
                    preview.variance_cents()
                ),
            });
        }
        let reconciled_txn_ids = self
            .reconciliation_transaction_ids_for(request.checking_account(), request.month_key());
        let matched_postings = i64::try_from(preview.matched_postings()).unwrap_or(i64::MAX);
        let (run, close) = self.store.write_reconciliation_run_and_month_close(
            request.month_key(),
            request.checking_account(),
            balances.opening_balance_cents,
            preview.ledger_delta_cents(),
            preview.expected_closing_balance_cents(),
            balances.closing_balance_cents,
            preview.variance_cents(),
            preview.is_reconciled(),
            matched_postings,
            preview.inflow_cents(),
            preview.outflow_cents(),
            &reconciled_txn_ids,
            request.analytics_artifact_id(),
        )?;
        let report = self.month_report_for(request.checking_account(), request.month_key());

        Ok(MonthAutopilotSummary::new(
            request.month_key(),
            request.checking_account(),
            imported_count,
            duplicate_count,
            fetch_runs,
            run,
            report,
            close,
        ))
    }

    #[allow(clippy::too_many_lines)]
    fn fetch_configured_statement_artifacts(
        &mut self,
        request: &MonthAutopilotRequest,
        fetch_required: bool,
    ) -> Result<Vec<StoredFetchRun>, RuntimeError> {
        let config = match self.load_statement_source_config() {
            Ok(config) => config,
            Err(_err) if !fetch_required => return Ok(Vec::new()),
            Err(err) => return Err(err),
        };
        let Some(config) = config else {
            return Ok(Vec::new());
        };

        let matched_sources: Vec<_> = config
            .sources()
            .iter()
            .filter(|source| source.ledger_account() == request.checking_account())
            .cloned()
            .collect();
        if matched_sources.is_empty() {
            return Ok(Vec::new());
        }

        let fetch_runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|err| RuntimeError::Analytics {
                message: format!("failed to start statement fetch runtime: {err}"),
            })?;

        let mut first_required_error = None;
        let mut staged_artifact = false;
        let mut persisted_runs = Vec::new();
        for source in matched_sources {
            let fetch_request = match FetchRequest::new(&source, request.month_key()) {
                Ok(fetch_request) => fetch_request,
                Err(err) => {
                    let err = fetch_error_to_runtime(&err);
                    if fetch_required && first_required_error.is_none() {
                        first_required_error = Some(RuntimeError::Analytics {
                            message: err.to_string(),
                        });
                    }
                    persisted_runs.push(self.persist_failed_fetch_run(
                        &source,
                        request.month_key(),
                        &err.to_string(),
                    )?);
                    continue;
                }
            };
            let secrets = match Self::secret_bundle_for_fetch_source(&source) {
                Ok(secrets) => secrets,
                Err(err) => {
                    if fetch_required && first_required_error.is_none() {
                        first_required_error = Some(RuntimeError::Analytics {
                            message: err.to_string(),
                        });
                    }
                    persisted_runs.push(self.persist_failed_fetch_run(
                        &source,
                        request.month_key(),
                        &err.to_string(),
                    )?);
                    continue;
                }
            };
            let result =
                match Self::run_fetch_adapter(&fetch_runtime, &source, &fetch_request, &secrets) {
                    Ok(result) => result,
                    Err(err) => {
                        if fetch_required && first_required_error.is_none() {
                            first_required_error = Some(RuntimeError::Analytics {
                                message: err.to_string(),
                            });
                        }
                        persisted_runs.push(self.persist_failed_fetch_run(
                            &source,
                            request.month_key(),
                            &err.to_string(),
                        )?);
                        continue;
                    }
                };
            if matches!(
                result.status(),
                FetchRunStatus::Downloaded | FetchRunStatus::Imported
            ) && result.artifact().is_none()
            {
                let message = format!(
                    "statement fetch for source '{}' returned {:?} without an artifact",
                    source.source_id(),
                    result.status()
                );
                if fetch_required && first_required_error.is_none() {
                    first_required_error = Some(RuntimeError::Analytics {
                        message: message.clone(),
                    });
                }
                persisted_runs.push(self.persist_failed_fetch_run(
                    &source,
                    request.month_key(),
                    &message,
                )?);
                continue;
            }
            match result.status() {
                FetchRunStatus::Downloaded | FetchRunStatus::Imported => {
                    if let Some(artifact) = result.artifact().cloned() {
                        self.stage_fetched_statement_artifact(artifact);
                        staged_artifact = true;
                    }
                    persisted_runs.push(self.persist_fetch_run_from_result(
                        &source,
                        request.month_key(),
                        &result,
                    )?);
                }
                FetchRunStatus::NoNewStatement => {
                    persisted_runs.push(self.persist_fetch_run_from_result(
                        &source,
                        request.month_key(),
                        &result,
                    )?);
                }
                FetchRunStatus::NeedsAttention | FetchRunStatus::Failed => {
                    persisted_runs.push(self.persist_fetch_run_from_result(
                        &source,
                        request.month_key(),
                        &result,
                    )?);
                    if fetch_required && first_required_error.is_none() {
                        first_required_error = Some(RuntimeError::Analytics {
                            message: format!(
                                "statement fetch for source '{}' requires attention before month autopilot can continue",
                                source.source_id()
                            ),
                        });
                    }
                }
            }
        }

        if fetch_required && !staged_artifact {
            if let Some(err) = first_required_error {
                return Err(err);
            }
        }

        Ok(persisted_runs)
    }

    fn fetched_statement_artifact_for(
        &self,
        checking_account: &str,
        month_key: &str,
    ) -> Option<FetchedStatementArtifact> {
        self.fetched_statement_artifacts
            .iter()
            .rev()
            .find(|artifact| {
                artifact.ledger_account() == checking_account && artifact.month_key() == month_key
            })
            .cloned()
    }

    fn import_fetched_statement_artifact(
        &mut self,
        artifact: &FetchedStatementArtifact,
        enable_ocr: bool,
    ) -> Result<(usize, usize), RuntimeError> {
        match artifact.output_format() {
            OutputFormat::Pdf => {
                let summary = self.import_pdf_statement(
                    artifact.artifact_path(),
                    artifact.ledger_account(),
                    false,
                    enable_ocr,
                )?;
                Ok((summary.imported_count(), summary.duplicate_count()))
            }
            OutputFormat::Csv => Err(RuntimeError::Analytics {
                message: format!(
                    "month autopilot cannot import fetched csv artifact '{}' yet",
                    artifact.artifact_path()
                ),
            }),
        }
    }

    fn resolve_autopilot_balances(
        &self,
        request: &MonthAutopilotRequest,
    ) -> Result<ResolvedAutopilotBalances, RuntimeError> {
        if let (Some(opening_balance_cents), Some(closing_balance_cents)) = (
            request.opening_balance_cents(),
            request.closing_balance_cents(),
        ) {
            return Ok(ResolvedAutopilotBalances {
                opening_balance_cents,
                closing_balance_cents,
            });
        }

        if let Some(artifact) =
            self.fetched_statement_artifact_for(request.checking_account(), request.month_key())
        {
            return Ok(ResolvedAutopilotBalances {
                opening_balance_cents: artifact.opening_balance_cents(),
                closing_balance_cents: artifact.closing_balance_cents(),
            });
        }

        Err(RuntimeError::Analytics {
            message:
                "month autopilot requires opening/closing balances or fetched statement metadata"
                    .to_owned(),
        })
    }

    fn load_statement_source_config(&self) -> Result<Option<StatementSourceConfig>, RuntimeError> {
        let Some(path) = self.fetch_config_path.as_ref() else {
            return Ok(None);
        };
        if !path.exists() {
            if env::var_os(LOGOS_FETCH_CONFIG_PATH_ENV).is_some() {
                return Err(RuntimeError::Analytics {
                    message: format!(
                        "statement fetch config path '{}' does not exist",
                        path.display()
                    ),
                });
            }
            return Ok(None);
        }

        let input = fs::read_to_string(path).map_err(|err| RuntimeError::Analytics {
            message: format!(
                "failed to read statement source config '{}': {err}",
                path.display()
            ),
        })?;

        StatementSourceConfig::from_toml(&input)
            .map(Some)
            .map_err(|err| fetch_error_to_runtime(&err))
    }

    fn run_fetch_adapter(
        fetch_runtime: &tokio::runtime::Runtime,
        source: &StatementSource,
        request: &FetchRequest,
        secrets: &SecretBundle,
    ) -> Result<logos_fetch::FetchResult, RuntimeError> {
        match source.institution_id() {
            "fake-fixture" => fetch_runtime
                .block_on(
                    FakeStatementAdapter::download_fixture_statement().fetch(request, secrets),
                )
                .map_err(|err| fetch_error_to_runtime(&err)),
            "fake-needs-attention" => fetch_runtime
                .block_on(
                    FakeStatementAdapter::needs_attention("mfa challenge required")
                        .fetch(request, secrets),
                )
                .map_err(|err| fetch_error_to_runtime(&err)),
            "provident-credit-union" => fetch_runtime
                .block_on(ProvidentAdapter::fixture_runner_output().fetch(request, secrets))
                .map_err(|err| fetch_error_to_runtime(&err)),
            institution_id => Err(RuntimeError::Analytics {
                message: format!(
                    "no statement fetch adapter is registered for institution \'{institution_id}\'"
                ),
            }),
        }
    }

    fn persist_fetch_run_from_result(
        &mut self,
        source: &StatementSource,
        month_key: &str,
        result: &logos_fetch::FetchResult,
    ) -> Result<StoredFetchRun, RuntimeError> {
        let artifact = result.artifact();
        self.store
            .write_fetch_run(
                source.source_id(),
                source.institution_id(),
                source.ledger_account(),
                month_key,
                store_fetch_run_status(result.status()),
                artifact.map(logos_fetch::FetchedStatementArtifact::artifact_path),
                artifact.map(|value| output_format_label(value.output_format())),
                artifact.map(logos_fetch::FetchedStatementArtifact::opening_balance_cents),
                artifact.map(logos_fetch::FetchedStatementArtifact::closing_balance_cents),
                result.error_summary(),
            )
            .map_err(RuntimeError::from)
    }

    fn persist_failed_fetch_run(
        &mut self,
        source: &StatementSource,
        month_key: &str,
        error_summary: &str,
    ) -> Result<StoredFetchRun, RuntimeError> {
        self.store
            .write_fetch_run(
                source.source_id(),
                source.institution_id(),
                source.ledger_account(),
                month_key,
                StoredFetchRunStatus::Failed,
                None,
                None,
                None,
                None,
                Some(error_summary),
            )
            .map_err(RuntimeError::from)
    }

    /// Projects a scenario-based RSU budget plan for a month scope.
    ///
    /// # Errors
    ///
    /// Returns an error when planning inputs are invalid.
    #[allow(clippy::too_many_arguments)]
    pub fn plan_rsu_budget_for_month(
        &self,
        month_key: &str,
        quarterly_units: u32,
        days_to_vest: u16,
        bear_price_cents: i64,
        base_price_cents: i64,
        bull_price_cents: i64,
        fixed_commitments_cents: i64,
        reserve_sweep_pct: u8,
        investing_sweep_pct: u8,
    ) -> Result<RsuBudgetPlan, RuntimeError> {
        let scenario_prices =
            ScenarioPriceInputs::new(bear_price_cents, base_price_cents, bull_price_cents)
                .map_err(|message| RuntimeError::Analytics { message })?;
        let input = RsuBudgetPlanInput::new(
            quarterly_units,
            days_to_vest,
            scenario_prices,
            fixed_commitments_cents,
            reserve_sweep_pct,
            investing_sweep_pct,
        )
        .map_err(|message| RuntimeError::Analytics { message })?;
        project_rsu_budget_plan(month_key, &input)
            .map_err(|message| RuntimeError::Analytics { message })
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
        let (content_hash_key, legacy_content_hash_key) = import_content_hash_keys(&record);
        if self.store.has_import_record_content_hash(&content_hash_key)
            || self
                .store
                .has_import_record_content_hash(&legacy_content_hash_key)
        {
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

    /// Imports transaction rows from a CSV statement file.
    ///
    /// When `dry_run` is true, rows are parsed and deduplicated but not posted.
    ///
    /// # Errors
    ///
    /// Returns an error when file IO, row parsing, or posting fails.
    pub fn import_csv_statement(
        &mut self,
        path: impl AsRef<Path>,
        mapping: &CsvMapping,
        dry_run: bool,
        skip_header: bool,
    ) -> Result<CsvImportSummary, RuntimeError> {
        let source_uri = path.as_ref().display().to_string();
        let csv_text = fs::read_to_string(&path).map_err(|err| ImportError::FileReadFailed {
            path: source_uri.clone(),
            message: err.to_string(),
        })?;

        let mut imported_count = 0_usize;
        let mut duplicate_count = 0_usize;

        // ⚡ Bolt Optimization: Pre-allocate collections to prevent repeated heap allocations
        // and re-hashing as records are imported.
        let estimated_lines = csv_text.lines().count();
        let mut seen_in_call: HashSet<String> = HashSet::with_capacity(estimated_lines);
        let mut imported_records = Vec::with_capacity(estimated_lines);
        let mut imported_keys = Vec::with_capacity(estimated_lines);

        for (line_idx, row) in csv_text.lines().enumerate() {
            if skip_header && line_idx == 0 {
                continue;
            }
            if row.trim().is_empty() {
                continue;
            }

            let record = parse_simple_csv_row(row, mapping)?;
            let (content_hash_key, legacy_content_hash_key) = import_content_hash_keys(&record);
            let seen_previously = self.store.has_import_record_content_hash(&content_hash_key)
                || self
                    .store
                    .has_import_record_content_hash(&legacy_content_hash_key);
            // Avoid allocating strings for duplicate records by checking existence first.
            let seen_in_batch = seen_in_call.contains(&content_hash_key);
            if seen_previously || seen_in_batch {
                duplicate_count = duplicate_count.saturating_add(1);
                continue;
            }
            seen_in_call.insert(content_hash_key.clone());

            imported_count = imported_count.saturating_add(1);
            if dry_run {
                continue;
            }

            let txn_id = self.post_import_record(&record)?;
            imported_records.push(NewImportRecord::with_statement_line(
                &content_hash_key,
                Some(&txn_id),
                &source_uri,
                record.timestamp(),
                record.memo(),
                record.amount_cents(),
            ));
            imported_keys.push(content_hash_key);
            self.imported_records = self.imported_records.saturating_add(1);
        }

        if !dry_run {
            let batch_key =
                import_batch_key("csv-statement", &source_uri, dry_run, false, &imported_keys);
            let duplicate_count = i64::try_from(duplicate_count).unwrap_or(i64::MAX);
            self.store.write_import_batch(
                "csv-statement",
                &source_uri,
                &batch_key,
                duplicate_count,
                dry_run,
                false,
                &imported_records,
            )?;
        }

        Ok(CsvImportSummary::new(
            imported_count,
            duplicate_count,
            dry_run,
        ))
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

        // ⚡ Bolt: Pre-allocate collections to prevent repeated heap allocations
        // and re-hashing as records are imported.
        let mut seen_in_call: HashSet<String> = HashSet::with_capacity(records.len());
        let mut imported_records = Vec::with_capacity(records.len());
        let mut imported_keys = Vec::with_capacity(records.len());

        for record in records {
            let (content_hash_key, legacy_content_hash_key) = import_content_hash_keys(&record);
            let seen_previously = self.store.has_import_record_content_hash(&content_hash_key)
                || self
                    .store
                    .has_import_record_content_hash(&legacy_content_hash_key);
            // Avoid allocating strings for duplicate records by checking existence first.
            let seen_in_batch = seen_in_call.contains(&content_hash_key);
            if seen_previously || seen_in_batch {
                duplicate_count = duplicate_count.saturating_add(1);
                continue;
            }
            seen_in_call.insert(content_hash_key.clone());

            imported_count = imported_count.saturating_add(1);
            if dry_run {
                continue;
            }

            let txn_id = self.post_import_record(&record)?;
            imported_records.push(NewImportRecord::with_statement_line(
                &content_hash_key,
                Some(&txn_id),
                &source_uri,
                record.timestamp(),
                record.memo(),
                record.amount_cents(),
            ));
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
    pub fn statement_lines_for_reconciliation_run(&self, run_id: &str) -> Vec<StoredStatementLine> {
        self.store.statement_lines_for_reconciliation_run(run_id)
    }

    /// Creates an immutable Parquet analytics snapshot and records its manifest in the store.
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
        let mut manifests = self.store.analytics_artifacts();
        manifests.sort_by(|left, right| {
            right
                .created_at()
                .cmp(&left.created_at())
                .then_with(|| left.artifact_id().cmp(right.artifact_id()))
        });
        manifests
    }

    #[must_use]
    pub fn get_analytics_snapshot(
        &self,
        artifact_id: &str,
    ) -> Option<StoredAnalyticsArtifactManifest> {
        self.store.analytics_artifact(artifact_id)
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
        let valid_from = parse_import_timestamp(record.timestamp());
        if amount > 0 {
            let builder =
                build_double_entry(record.memo(), record.account(), record.category(), amount)?;
            return self
                .store
                .write_transaction_with_valid_time(builder, valid_from)
                .map_err(RuntimeError::from);
        }

        let debit_amount = amount.checked_abs().ok_or(ImportError::InvalidAmount)?;
        let builder = build_double_entry(
            record.memo(),
            record.category(),
            record.account(),
            debit_amount,
        )?;
        self.store
            .write_transaction_with_valid_time(builder, valid_from)
            .map_err(RuntimeError::from)
    }

    fn expense_total_for_month(&self, month_key: &str, expense_account_prefix: &str) -> i64 {
        let mut total = 0_i64;
        for stored in self
            .store
            .transactions()
            .into_iter()
            .filter(|stored| transaction_in_month(stored, month_key))
        {
            for posting in stored.transaction().postings() {
                if posting
                    .account()
                    .as_str()
                    .starts_with(expense_account_prefix)
                {
                    let amount = posting.amount();
                    if amount > 0 {
                        total = total.saturating_add(amount);
                    }
                }
            }
        }

        total
    }

    fn reconciliation_transaction_ids_for(
        &self,
        checking_account: &str,
        month_key: &str,
    ) -> Vec<TransactionId> {
        let mut ids: Vec<_> = self
            .store
            .transactions()
            .into_iter()
            .filter(|stored| transaction_in_month(stored, month_key))
            .filter(|stored| {
                stored
                    .transaction()
                    .postings()
                    .iter()
                    .any(|posting| posting.account().as_str() == checking_account)
            })
            .map(|stored| stored.id().clone())
            .collect();
        ids.sort_by(|left, right| left.as_str().cmp(right.as_str()));
        ids.dedup_by(|left, right| left.as_str() == right.as_str());
        ids
    }
}

fn import_content_hash_key_legacy_v1(fingerprint: u64) -> String {
    format!("{fingerprint:016x}")
}

fn import_content_hash_keys(record: &ImportRecord) -> (String, String) {
    let content_hash_key = deterministic_fingerprint(record);
    // Backward compatibility: detect duplicates imported before the v2 fingerprint rollout.
    let legacy_content_hash_key =
        import_content_hash_key_legacy_v1(deterministic_fingerprint_legacy_v1(record));
    (content_hash_key, legacy_content_hash_key)
}

fn import_batch_key(
    import_kind: &str,
    source_uri: &str,
    dry_run: bool,
    ocr_enabled: bool,
    imported_keys: &[String],
) -> String {
    let mut sorted_keys: Vec<&str> = imported_keys.iter().map(String::as_str).collect();
    sorted_keys.sort_unstable();

    let mut hasher = Hasher::new();
    hasher.update(b"kind:");
    hasher.update(import_kind.as_bytes());
    hasher.update(b"|source:");
    hasher.update(source_uri.as_bytes());
    hasher.update(b"|dry_run:");
    hasher.update(if dry_run { b"true" } else { b"false" });
    hasher.update(b"|ocr:");
    hasher.update(if ocr_enabled { b"true\n" } else { b"false\n" });
    for key in sorted_keys {
        hasher.update(key.as_bytes());
        hasher.update(b"\n");
    }
    hasher.finalize().to_hex().to_string()
}

#[must_use]
pub fn default_artifacts_root(state_root: &Path) -> PathBuf {
    if let Some(path) = env::var_os(LOGOS_ARTIFACTS_PATH_ENV) {
        return PathBuf::from(path);
    }

    state_root.join(ARTIFACTS_DIRECTORY)
}

#[must_use]
pub fn default_fetch_config_path(state_root: &Path) -> PathBuf {
    if let Some(path) = env::var_os(LOGOS_FETCH_CONFIG_PATH_ENV) {
        return PathBuf::from(path);
    }

    state_root.join(DEFAULT_FETCH_CONFIG_NAME)
}

#[must_use]
pub fn default_state_root() -> PathBuf {
    if let Some(home) = env::var_os("HOME").or_else(|| env::var_os("USERPROFILE")) {
        return PathBuf::from(home).join(DEFAULT_STATE_DIRECTORY);
    }

    env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(DEFAULT_STATE_DIRECTORY)
}

fn fetch_error_to_runtime(err: &logos_fetch::FetchError) -> RuntimeError {
    RuntimeError::Analytics {
        message: err.to_string(),
    }
}

const fn output_format_label(output_format: OutputFormat) -> StoredFetchArtifactFormat {
    match output_format {
        OutputFormat::Csv => StoredFetchArtifactFormat::Csv,
        OutputFormat::Pdf => StoredFetchArtifactFormat::Pdf,
    }
}

const fn store_fetch_run_status(status: FetchRunStatus) -> StoredFetchRunStatus {
    match status {
        FetchRunStatus::Downloaded => StoredFetchRunStatus::Downloaded,
        FetchRunStatus::Imported => StoredFetchRunStatus::Imported,
        FetchRunStatus::NoNewStatement => StoredFetchRunStatus::NoNewStatement,
        FetchRunStatus::NeedsAttention => StoredFetchRunStatus::NeedsAttention,
        FetchRunStatus::Failed => StoredFetchRunStatus::Failed,
    }
}

fn current_time_us() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_micros())
        .ok()
        .and_then(|micros| i64::try_from(micros).ok())
        .unwrap_or(0)
}

/// Extracts `SnapshotPostingRow`s from a list of transactions.
///
/// **Performance Optimization**: Pre-calculates the exact number of postings across all
/// transactions to pre-allocate the `rows` vector. This avoids multiple intermediate
/// heap reallocations when processing large ledgers.
fn snapshot_rows(transactions: Vec<StoredTransaction>) -> Vec<SnapshotPostingRow> {
    let capacity: usize = transactions
        .iter()
        .map(|stored| stored.transaction().postings().len())
        .sum();
    let mut rows = Vec::with_capacity(capacity);
    for stored in transactions {
        for (index, posting) in stored.transaction().postings().iter().enumerate() {
            let posting_ordinal = i64::try_from(index).unwrap_or(i64::MAX);
            rows.push(SnapshotPostingRow {
                txn_id: stored.id().as_str().to_owned(),
                description: stored.transaction().description().to_owned(),
                effective_at_us: stored.effective_at(),
                posting_ordinal,
                account: posting.account().as_str().to_owned(),
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

/// Hashes the snapshot rows for parquet metadata generation.
///
/// ⚡ Bolt Optimization: Replaces `hasher.update(format!(...).as_bytes())` with `writeln!`.
/// This prevents allocating a new `String` on the heap for every single row in the snapshot,
/// streaming the formatted bytes directly into the `blake3::Hasher`. For a ledger with 100k
/// postings, this removes 100k+ unnecessary heap allocations during snapshot generation.
fn hash_rows(
    rows: &[SnapshotPostingRow],
    as_of_valid: i64,
    as_of_tx: i64,
    schema_version: i64,
) -> String {
    let mut hasher = Hasher::new();
    writeln!(
        &mut hasher,
        "schema:{schema_version}|valid:{as_of_valid}|tx:{as_of_tx}"
    )
    .expect("writeln! to Hasher should never fail");
    for row in rows {
        writeln!(
            &mut hasher,
            "{}|{}|{}|{}|{}|{}",
            row.txn_id,
            row.description,
            row.effective_at_us,
            row.posting_ordinal,
            row.account,
            row.amount_cents
        )
        .expect("writeln! to Hasher should never fail");
    }
    hasher.finalize().to_hex().to_string()
}

/// Writes ledger snapshot rows to a Parquet file.
///
/// ⚡ Bolt Optimization: Eliminates 8 intermediate heap allocations by collecting
/// directly from the `rows` slice iterators into Polars `Series` objects using
/// `FromIterator`, rather than allocating intermediate `Vec<T>`s first.
fn write_rows_to_parquet(
    parquet_path: &Path,
    rows: &[SnapshotPostingRow],
    as_of_valid: i64,
    as_of_tx: i64,
) -> Result<(), RuntimeError> {
    let mut frame = DataFrame::new(vec![
        rows.iter()
            .map(|row| row.txn_id.as_str())
            .collect::<Series>()
            .with_name("txn_id".into())
            .into(),
        rows.iter()
            .map(|row| row.description.as_str())
            .collect::<Series>()
            .with_name("description".into())
            .into(),
        rows.iter()
            .map(|row| row.effective_at_us)
            .collect::<Series>()
            .with_name("effective_at_us".into())
            .into(),
        rows.iter()
            .map(|row| row.posting_ordinal)
            .collect::<Series>()
            .with_name("posting_ordinal".into())
            .into(),
        rows.iter()
            .map(|row| row.account.as_str())
            .collect::<Series>()
            .with_name("account".into())
            .into(),
        rows.iter()
            .map(|row| row.amount_cents)
            .collect::<Series>()
            .with_name("amount_cents".into())
            .into(),
        std::iter::repeat_n(as_of_valid, rows.len())
            .collect::<Series>()
            .with_name("snapshot_valid_at_us".into())
            .into(),
        std::iter::repeat_n(as_of_tx, rows.len())
            .collect::<Series>()
            .with_name("snapshot_tx_at_us".into())
            .into(),
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

fn build_double_entry(
    description: &str,
    debit_account: &str,
    credit_account: &str,
    amount_cents: i64,
) -> Result<TransactionBuilder, RuntimeError> {
    use logos_core::AccountId;
    Ok(TransactionBuilder::new(description)
        .posting(Posting::debit(
            AccountId::new(debit_account)?,
            amount_cents,
        )?)
        .posting(Posting::credit(
            AccountId::new(credit_account)?,
            amount_cents,
        )?))
}

fn parse_import_timestamp(timestamp: &str) -> Option<i64> {
    let trimmed = timestamp.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Ok(dt) = DateTime::parse_from_rfc3339(trimmed) {
        return Some(dt.timestamp_micros());
    }
    if let Ok(dt) = NaiveDateTime::parse_from_str(trimmed, "%Y-%m-%dT%H:%M:%S") {
        return Some(DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc).timestamp_micros());
    }
    if let Ok(date) = NaiveDate::parse_from_str(trimmed, "%Y-%m-%d") {
        let date_time = date.and_hms_opt(0, 0, 0)?;
        return Some(DateTime::<Utc>::from_naive_utc_and_offset(date_time, Utc).timestamp_micros());
    }
    if let Ok(micros) = trimmed.parse::<i64>() {
        return Some(micros);
    }

    None
}

fn transaction_in_month(stored: &StoredTransaction, month_key: &str) -> bool {
    month_key_from_wallclock_utc(stored.effective_at()) == month_key
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

#[cfg(test)]
mod tests {
    use super::AppRuntime;
    use logos_fetch::{FetchError, OutputFormat, SecretBundle, SecretResolver, StatementSource};

    #[derive(Debug, Clone)]
    struct StubSecretResolver {
        bundle: SecretBundle,
    }

    impl SecretResolver for StubSecretResolver {
        fn resolve(&self, _source: &StatementSource) -> Result<SecretBundle, FetchError> {
            Ok(self.bundle.clone())
        }
    }

    #[test]
    fn fake_fetch_sources_bypass_external_secret_resolution() {
        let source = StatementSource::new(
            "fixture:checking",
            "fake-fixture",
            "assets:checking",
            vec![OutputFormat::Pdf],
        )
        .expect("source");
        let resolver = StubSecretResolver {
            bundle: SecretBundle::new("wrong", "wrong", Some("999999")).expect("bundle"),
        };

        let bundle = AppRuntime::<logos_store::memory::MemoryStore>::secret_bundle_for_fetch_source_with_resolver(&source, &resolver)
            .expect("bundle");

        assert_eq!(
            bundle,
            SecretBundle::new("fixture-user", "fixture-pass", Some("000000")).expect("fixture")
        );
    }

    #[test]
    fn real_fetch_sources_use_external_secret_resolution() {
        let source = StatementSource::new(
            "pcu:checking",
            "provident-credit-union",
            "assets:checking",
            vec![OutputFormat::Pdf],
        )
        .expect("source")
        .with_secret_refs(
            "op://logos/provident/username",
            "op://logos/provident/password",
            Some("op://logos/provident/totp"),
        )
        .expect("secret refs");
        let resolver = StubSecretResolver {
            bundle: SecretBundle::new("markm", "s3cr3t", Some("123456")).expect("bundle"),
        };

        let bundle = AppRuntime::<logos_store::memory::MemoryStore>::secret_bundle_for_fetch_source_with_resolver(&source, &resolver)
            .expect("bundle");

        assert_eq!(
            bundle,
            SecretBundle::new("markm", "s3cr3t", Some("123456")).expect("expected")
        );
    }
}
