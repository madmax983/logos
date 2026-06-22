//! The Memory Store
//!
//! An ephemeral, hashmap-backed implementation of the [`LedgerStore`] trait.
//! Strictly meant for rapid testing, doctests, and isolated environment simulation where persistence is not required.

use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;
use std::time::{SystemTime, UNIX_EPOCH};

use logos_core::{Correction, TransactionBuilder, TransactionId};

use crate::error::StoreError;
use crate::model::{
    BudgetTargetKey, NewImportRecord, StoredAnalyticsArtifactManifest, StoredBudgetTarget,
    StoredCorrection, StoredFetchArtifactFormat, StoredFetchRun, StoredFetchRunStatus,
    StoredImportBatch, StoredImportRecord, StoredMonthClose, StoredReconciliationRun,
    StoredStatementLine, StoredTransaction, Timestamp,
};
use crate::traits::LedgerStore;

#[derive(Debug)]
/// An in-memory implementation of the [`LedgerStore`] trait for testing and simulations.
///
/// Stores all entities in memory. Data is lost as soon as the instance is dropped.
///
/// ## Examples
///
/// ```
/// use logos_store::{LedgerStore, MemoryStore};
///
/// let mut store = MemoryStore::new();
/// assert_eq!(store.budget_targets().len(), 0);
/// ```
pub struct MemoryStore {
    clock_us: Timestamp,
    next_transaction_id: u64,
    next_artifact_id: u64,
    next_import_batch_id: u64,
    next_statement_line_id: u64,
    next_fetch_run_id: u64,
    next_reconciliation_run_id: u64,
    next_month_close_id: u64,
    transactions: HashMap<TransactionId, TransactionEntry>,
    corrections: Vec<CorrectionEntry>,
    budget_targets: HashMap<BudgetTargetKey, StoredBudgetTarget>,
    analytics_artifacts: HashMap<String, StoredAnalyticsArtifactManifest>,
    import_batches: HashMap<String, StoredImportBatch>,
    import_records: HashMap<String, StoredImportRecord>,
    statement_lines: HashMap<String, StoredStatementLine>,
    statement_line_ids_by_txn: HashMap<TransactionId, Vec<String>>,
    fetch_runs: HashMap<String, StoredFetchRun>,
    reconciliation_runs: HashMap<String, StoredReconciliationRun>,
    reconciliation_statement_line_ids: HashMap<String, Vec<String>>,
    month_closes: HashMap<String, StoredMonthClose>,
    month_close_by_scope: HashMap<BudgetTargetKey, String>,
}

#[derive(Debug, Clone)]
struct TransactionEntry {
    stored: StoredTransaction,
    recorded_at_us: Timestamp,
}

#[derive(Debug, Clone)]
struct CorrectionEntry {
    stored: StoredCorrection,
    recorded_at_us: Timestamp,
}

impl Default for MemoryStore {
    fn default() -> Self {
        Self {
            clock_us: system_time_us(),
            next_transaction_id: 0,
            next_artifact_id: 0,
            next_import_batch_id: 0,
            next_statement_line_id: 0,
            next_fetch_run_id: 0,
            next_reconciliation_run_id: 0,
            next_month_close_id: 0,
            transactions: HashMap::new(),
            corrections: Vec::new(),
            budget_targets: HashMap::new(),
            analytics_artifacts: HashMap::new(),
            import_batches: HashMap::new(),
            import_records: HashMap::new(),
            statement_lines: HashMap::new(),
            statement_line_ids_by_txn: HashMap::new(),
            fetch_runs: HashMap::new(),
            reconciliation_runs: HashMap::new(),
            reconciliation_statement_line_ids: HashMap::new(),
            month_closes: HashMap::new(),
            month_close_by_scope: HashMap::new(),
        }
    }
}

impl MemoryStore {
    /// Creates a new, empty in-memory store.
    ///
    /// ## Examples
    ///
    /// ```
    /// use logos_store::MemoryStore;
    /// let store = MemoryStore::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new, empty in-memory store. This is an alias for `new()`.
    ///
    /// ## Examples
    ///
    /// ```
    /// use logos_store::MemoryStore;
    /// let store = MemoryStore::new_in_memory();
    /// ```
    #[must_use]
    pub fn new_in_memory() -> Self {
        Self::default()
    }

    const fn next_timestamp_us(&mut self) -> Timestamp {
        self.clock_us = self.clock_us.saturating_add(1);
        self.clock_us
    }

    fn next_transaction_id(&mut self) -> TransactionId {
        self.next_transaction_id = self.next_transaction_id.saturating_add(1);
        TransactionId::new(&format!("txn-{}", self.next_transaction_id))
            .expect("generated transaction id is always valid")
    }

    fn next_artifact_id(&mut self) -> String {
        self.next_artifact_id = self.next_artifact_id.saturating_add(1);
        format!("artifact-{}", self.next_artifact_id)
    }

    fn next_import_batch_id(&mut self) -> String {
        self.next_import_batch_id = self.next_import_batch_id.saturating_add(1);
        format!("import-batch-{}", self.next_import_batch_id)
    }

    fn next_statement_line_id(&mut self) -> String {
        self.next_statement_line_id = self.next_statement_line_id.saturating_add(1);
        format!("stmt-line-{}", self.next_statement_line_id)
    }

    fn next_fetch_run_id(&mut self) -> String {
        self.next_fetch_run_id = self.next_fetch_run_id.saturating_add(1);
        format!("fetch-{}", self.next_fetch_run_id)
    }

    fn next_reconciliation_run_id(&mut self) -> String {
        self.next_reconciliation_run_id = self.next_reconciliation_run_id.saturating_add(1);
        format!("recon-{}", self.next_reconciliation_run_id)
    }

    fn next_month_close_id(&mut self) -> String {
        self.next_month_close_id = self.next_month_close_id.saturating_add(1);
        format!("close-{}", self.next_month_close_id)
    }

    fn transaction_exists(&self, id: &TransactionId) -> bool {
        self.transactions.contains_key(id)
    }

    fn current_transactions(&self) -> Vec<StoredTransaction> {
        let superseded = self.superseded_transaction_ids(i64::MAX);
        let mut rows: Vec<_> = self
            .transactions
            .values()
            .filter(|entry| !superseded.contains(entry.stored.id()))
            .map(|entry| entry.stored.clone())
            .collect();
        rows.sort_by(|left, right| left.id().as_str().cmp(right.id().as_str()));
        rows
    }

    fn superseded_transaction_ids(&self, tx_time_us: Timestamp) -> HashSet<TransactionId> {
        self.corrections
            .iter()
            .filter(|entry| entry.recorded_at_us <= tx_time_us)
            .map(|entry| entry.stored.correction().supersedes_id().clone())
            .collect()
    }

    fn transactions_as_of_inner(
        &self,
        valid_time_us: Timestamp,
        tx_time_us: Timestamp,
    ) -> Vec<StoredTransaction> {
        let superseded = self.superseded_transaction_ids(tx_time_us);
        let mut rows: Vec<_> = self
            .transactions
            .values()
            .filter(|entry| {
                entry.recorded_at_us <= tx_time_us
                    && entry.stored.effective_at() <= valid_time_us
                    && !superseded.contains(entry.stored.id())
            })
            .map(|entry| entry.stored.clone())
            .collect();
        rows.sort_by(|left, right| left.id().as_str().cmp(right.id().as_str()));
        rows
    }

    fn statement_line_ids_for_transaction_ids(
        &self,
        transaction_ids: &[TransactionId],
    ) -> Vec<String> {
        // ⚡ Bolt Optimization: Avoid HashSet allocation and use flat_map
        let mut line_ids: Vec<_> = transaction_ids
            .iter()
            .filter_map(|txn_id| self.statement_line_ids_by_txn.get(txn_id))
            .flat_map(|line_ids| line_ids.iter().cloned())
            .collect();
        line_ids.sort();
        line_ids.dedup();
        line_ids
    }

    fn store_transaction(
        &mut self,
        id: TransactionId,
        transaction: logos_core::Transaction,
        effective_at: Timestamp,
        recorded_at_us: Timestamp,
    ) {
        self.transactions.insert(
            id.clone(),
            TransactionEntry {
                stored: StoredTransaction::with_effective_at(id, transaction, effective_at),
                recorded_at_us,
            },
        );
    }
}

impl LedgerStore for MemoryStore {
    fn transaction_count(&self) -> usize {
        self.transactions.len()
    }

    fn correction_count(&self) -> usize {
        self.corrections.len()
    }

    fn has_transaction(&self, id: &TransactionId) -> bool {
        self.transaction_exists(id)
    }

    fn latest_correction(&self) -> Option<Correction> {
        self.corrections
            .last()
            .map(|entry| entry.stored.correction().clone())
    }

    fn transactions(&self) -> Vec<StoredTransaction> {
        self.current_transactions()
    }

    fn budget_target(
        &self,
        month_key: &str,
        expense_account_prefix: &str,
    ) -> Option<StoredBudgetTarget> {
        self.budget_targets
            .get(&(month_key.to_owned(), expense_account_prefix.to_owned()))
            .cloned()
    }

    fn budget_targets(&self) -> Vec<StoredBudgetTarget> {
        let mut rows: Vec<_> = self.budget_targets.values().cloned().collect();
        rows.sort_by(|left, right| {
            left.month_key().cmp(right.month_key()).then_with(|| {
                left.expense_account_prefix()
                    .cmp(right.expense_account_prefix())
            })
        });
        rows
    }

    fn analytics_artifact(&self, artifact_id: &str) -> Option<StoredAnalyticsArtifactManifest> {
        self.analytics_artifacts.get(artifact_id).cloned()
    }

    fn analytics_artifacts(&self) -> Vec<StoredAnalyticsArtifactManifest> {
        let mut rows: Vec<_> = self.analytics_artifacts.values().cloned().collect();
        rows.sort_by(|left, right| left.artifact_id().cmp(right.artifact_id()));
        rows
    }

    fn import_record_count(&self) -> usize {
        self.import_records.len()
    }

    fn has_import_record_content_hash(&self, content_hash_key: &str) -> bool {
        self.import_records.contains_key(content_hash_key)
    }

    fn import_records(&self) -> Vec<StoredImportRecord> {
        let mut rows: Vec<_> = self.import_records.values().cloned().collect();
        rows.sort_by(|left, right| left.content_hash_key().cmp(right.content_hash_key()));
        rows
    }

    fn import_batches(&self) -> Vec<StoredImportBatch> {
        let mut rows: Vec<_> = self.import_batches.values().cloned().collect();
        rows.sort_by(|left, right| left.batch_id().cmp(right.batch_id()));
        rows
    }

    fn statement_line_count(&self) -> usize {
        self.statement_lines.len()
    }

    fn statement_lines(&self) -> Vec<StoredStatementLine> {
        let mut rows: Vec<_> = self.statement_lines.values().cloned().collect();
        rows.sort_by(|left, right| left.line_id().cmp(right.line_id()));
        rows
    }

    fn fetch_run_count(&self) -> usize {
        self.fetch_runs.len()
    }

    fn fetch_run(&self, run_id: &str) -> Option<StoredFetchRun> {
        self.fetch_runs.get(run_id).cloned()
    }

    fn fetch_runs(&self) -> Vec<StoredFetchRun> {
        let mut rows: Vec<_> = self.fetch_runs.values().cloned().collect();
        rows.sort_by(|left, right| left.run_id().cmp(right.run_id()));
        rows
    }

    fn statement_lines_for_reconciliation_run(&self, run_id: &str) -> Vec<StoredStatementLine> {
        let Some(line_ids) = self.reconciliation_statement_line_ids.get(run_id) else {
            return Vec::new();
        };

        let mut rows: Vec<_> = line_ids
            .iter()
            .filter_map(|line_id| self.statement_lines.get(line_id).cloned())
            .collect();
        rows.sort_by(|left, right| left.line_id().cmp(right.line_id()));
        rows
    }

    fn reconciliation_run_count(&self) -> usize {
        self.reconciliation_runs.len()
    }

    fn reconciliation_run(&self, run_id: &str) -> Option<StoredReconciliationRun> {
        self.reconciliation_runs.get(run_id).cloned()
    }

    fn reconciliation_runs(&self) -> Vec<StoredReconciliationRun> {
        let mut rows: Vec<_> = self.reconciliation_runs.values().cloned().collect();
        rows.sort_by(|left, right| left.run_id().cmp(right.run_id()));
        rows
    }

    fn month_close_count(&self) -> usize {
        self.month_closes.len()
    }

    fn month_close(&self, close_id: &str) -> Option<StoredMonthClose> {
        self.month_closes.get(close_id).cloned()
    }

    fn month_close_for_scope(
        &self,
        month_key: &str,
        checking_account: &str,
    ) -> Option<StoredMonthClose> {
        let key = (month_key.to_owned(), checking_account.to_owned());
        self.month_close_by_scope
            .get(&key)
            .and_then(|close_id| self.month_closes.get(close_id))
            .cloned()
    }

    fn month_closes(&self) -> Vec<StoredMonthClose> {
        let mut rows: Vec<_> = self.month_closes.values().cloned().collect();
        rows.sort_by(|left, right| left.close_id().cmp(right.close_id()));
        rows
    }

    fn transactions_as_of_us(
        &self,
        valid_time_us: i64,
        tx_time_us: i64,
    ) -> Result<Vec<StoredTransaction>, StoreError> {
        Ok(self.transactions_as_of_inner(valid_time_us, tx_time_us))
    }

    fn write_transaction(
        &mut self,
        builder: TransactionBuilder,
    ) -> Result<TransactionId, StoreError> {
        self.write_transaction_with_valid_time(builder, None)
    }

    fn write_transaction_with_valid_time(
        &mut self,
        builder: TransactionBuilder,
        valid_from: Option<i64>,
    ) -> Result<TransactionId, StoreError> {
        let transaction = builder.build()?;
        let recorded_at_us = self.next_timestamp_us();
        let effective_at = valid_from.unwrap_or(recorded_at_us);
        let transaction_id = self.next_transaction_id();
        self.store_transaction(
            transaction_id.clone(),
            transaction,
            effective_at,
            recorded_at_us,
        );
        Ok(transaction_id)
    }

    fn write_correction(&mut self, correction: Correction) -> Result<(), StoreError> {
        if !self.transaction_exists(correction.supersedes_id()) {
            return Err(StoreError::UnknownTransaction {
                transaction_id: correction.into_supersedes_id(),
            });
        }

        let recorded_at_us = self.next_timestamp_us();
        self.corrections.push(CorrectionEntry {
            stored: StoredCorrection::new(correction),
            recorded_at_us,
        });
        Ok(())
    }

    fn write_budget_target(
        &mut self,
        month_key: &str,
        expense_account_prefix: &str,
        budget_cents: i64,
    ) -> Result<(), StoreError> {
        let target = StoredBudgetTarget::new(month_key, expense_account_prefix, budget_cents);
        self.budget_targets.insert(
            (month_key.to_owned(), expense_account_prefix.to_owned()),
            target,
        );
        Ok(())
    }

    fn write_analytics_artifact_manifest(
        &mut self,
        artifact_kind: &str,
        artifact_uri: &str,
        content_hash: &str,
        schema_version: i64,
        row_count: i64,
        snapshot_valid_at_us: i64,
        snapshot_tx_at_us: i64,
        supersedes_artifact_id: Option<&str>,
    ) -> Result<StoredAnalyticsArtifactManifest, StoreError> {
        self.write_analytics_artifact_manifest_us(
            artifact_kind,
            artifact_uri,
            content_hash,
            schema_version,
            row_count,
            snapshot_valid_at_us,
            snapshot_tx_at_us,
            supersedes_artifact_id,
        )
    }

    fn write_analytics_artifact_manifest_us(
        &mut self,
        artifact_kind: &str,
        artifact_uri: &str,
        content_hash: &str,
        schema_version: i64,
        row_count: i64,
        snapshot_valid_at_us: i64,
        snapshot_tx_at_us: i64,
        supersedes_artifact_id: Option<&str>,
    ) -> Result<StoredAnalyticsArtifactManifest, StoreError> {
        if let Some(artifact_id) = supersedes_artifact_id
            && !self.analytics_artifacts.contains_key(artifact_id)
        {
            return Err(StoreError::UnknownArtifact {
                artifact_id: artifact_id.to_owned(),
            });
        }

        let artifact_id = self.next_artifact_id();
        let created_at = self.next_timestamp_us();
        let snapshot_key = format!("valid:{snapshot_valid_at_us}|tx:{snapshot_tx_at_us}");
        let manifest = StoredAnalyticsArtifactManifest::new(
            &artifact_id,
            artifact_kind,
            artifact_uri,
            content_hash,
            schema_version,
            row_count,
            snapshot_valid_at_us,
            snapshot_tx_at_us,
            created_at,
            supersedes_artifact_id,
            &snapshot_key,
        );
        self.analytics_artifacts
            .insert(artifact_id, manifest.clone());
        Ok(manifest)
    }

    fn write_import_batch(
        &mut self,
        import_kind: &str,
        source_uri: &str,
        batch_key: &str,
        duplicate_count: i64,
        dry_run: bool,
        ocr_enabled: bool,
        records: &[NewImportRecord],
    ) -> Result<StoredImportBatch, StoreError> {
        if import_kind.is_empty() {
            return Err(StoreError::PersistFailed {
                message: "import_kind must not be empty".to_owned(),
            });
        }
        if source_uri.is_empty() {
            return Err(StoreError::PersistFailed {
                message: "source_uri must not be empty".to_owned(),
            });
        }
        if batch_key.is_empty() {
            return Err(StoreError::PersistFailed {
                message: "batch_key must not be empty".to_owned(),
            });
        }
        if duplicate_count < 0 {
            return Err(StoreError::PersistFailed {
                message: format!("duplicate_count must be non-negative, got {duplicate_count}"),
            });
        }

        let mut seen_keys = HashSet::new();
        for record in records {
            let key = record.content_hash_key();
            if key.is_empty() {
                return Err(StoreError::PersistFailed {
                    message: "import record content_hash_key must not be empty".to_owned(),
                });
            }
            if !seen_keys.insert(key.to_owned()) {
                return Err(StoreError::PersistFailed {
                    message: format!(
                        "duplicate import record content_hash_key '{key}' in write payload"
                    ),
                });
            }
            if self.import_records.contains_key(key) {
                return Err(StoreError::PersistFailed {
                    message: format!("import record content_hash_key '{key}' already exists"),
                });
            }
        }

        let imported_txn_ids: Vec<TransactionId> = records
            .iter()
            .filter_map(|record| record.imported_txn_id().cloned())
            .collect();
        for txn_id in &imported_txn_ids {
            if !self.transaction_exists(txn_id) {
                return Err(StoreError::UnknownTransaction {
                    transaction_id: txn_id.clone(),
                });
            }
        }

        let imported_at = self.next_timestamp_us();
        let batch_id = self.next_import_batch_id();
        let record_count = i64::try_from(records.len()).unwrap_or(i64::MAX);
        let batch = StoredImportBatch::new(
            &batch_id,
            import_kind,
            source_uri,
            batch_key,
            record_count,
            duplicate_count,
            dry_run,
            ocr_enabled,
            imported_at,
        );

        for record in records {
            let stored_record = StoredImportRecord::new(
                record.content_hash_key(),
                &batch_id,
                record.imported_txn_id().cloned(),
                imported_at,
            );
            self.import_records
                .insert(stored_record.content_hash_key().to_owned(), stored_record);

            if let Some(line) = record.statement_line() {
                let line_id = self.next_statement_line_id();
                let stored_line = StoredStatementLine::new(
                    &line_id,
                    &batch_id,
                    line.source_uri(),
                    line.statement_timestamp(),
                    line.memo(),
                    line.amount_cents(),
                    record.imported_txn_id().cloned(),
                    imported_at,
                );
                if let Some(txn_id) = record.imported_txn_id().cloned() {
                    self.statement_line_ids_by_txn
                        .entry(txn_id)
                        .or_default()
                        .push(line_id.clone());
                }
                self.statement_lines.insert(line_id, stored_line);
            }
        }

        self.import_batches.insert(batch_id, batch.clone());
        Ok(batch)
    }

    fn write_fetch_run(
        &mut self,
        source_id: &str,
        institution_id: &str,
        ledger_account: &str,
        month_key: &str,
        status: StoredFetchRunStatus,
        artifact_path: Option<&str>,
        output_format: Option<StoredFetchArtifactFormat>,
        opening_balance_cents: Option<i64>,
        closing_balance_cents: Option<i64>,
        error_summary: Option<&str>,
    ) -> Result<StoredFetchRun, StoreError> {
        if source_id.is_empty() {
            return Err(StoreError::PersistFailed {
                message: "source_id must not be empty".to_owned(),
            });
        }
        if institution_id.is_empty() {
            return Err(StoreError::PersistFailed {
                message: "institution_id must not be empty".to_owned(),
            });
        }
        if ledger_account.is_empty() {
            return Err(StoreError::PersistFailed {
                message: "ledger_account must not be empty".to_owned(),
            });
        }
        if month_key.is_empty() {
            return Err(StoreError::PersistFailed {
                message: "month_key must not be empty".to_owned(),
            });
        }
        if matches!(
            status,
            StoredFetchRunStatus::Downloaded | StoredFetchRunStatus::Imported
        ) && (artifact_path.is_none()
            || output_format.is_none()
            || opening_balance_cents.is_none()
            || closing_balance_cents.is_none())
        {
            return Err(StoreError::PersistFailed {
                message: format!(
                    "status '{}' requires artifact path, format, and balance metadata",
                    status.as_str()
                ),
            });
        }

        let created_at = self.next_timestamp_us();
        let run_id = self.next_fetch_run_id();
        let normalized_error_summary = error_summary.filter(|value| !value.is_empty());
        let run = StoredFetchRun::new(
            &run_id,
            source_id,
            institution_id,
            ledger_account,
            month_key,
            status,
            artifact_path,
            output_format,
            opening_balance_cents,
            closing_balance_cents,
            normalized_error_summary,
            created_at,
        );
        self.fetch_runs.insert(run_id, run.clone());
        Ok(run)
    }

    fn write_reconciliation_run(
        &mut self,
        month_key: &str,
        checking_account: &str,
        opening_balance_cents: i64,
        ledger_delta_cents: i64,
        expected_closing_balance_cents: i64,
        statement_closing_balance_cents: i64,
        variance_cents: i64,
        reconciled: bool,
        matched_postings: i64,
        inflow_cents: i64,
        outflow_cents: i64,
        reconciled_txn_ids: &[TransactionId],
    ) -> Result<StoredReconciliationRun, StoreError> {
        if month_key.is_empty() {
            return Err(StoreError::PersistFailed {
                message: "month_key must not be empty".to_owned(),
            });
        }
        if checking_account.is_empty() {
            return Err(StoreError::PersistFailed {
                message: "checking_account must not be empty".to_owned(),
            });
        }
        if matched_postings < 0 {
            return Err(StoreError::PersistFailed {
                message: format!("matched_postings must be non-negative, got {matched_postings}"),
            });
        }
        if inflow_cents < 0 {
            return Err(StoreError::PersistFailed {
                message: format!("inflow_cents must be non-negative, got {inflow_cents}"),
            });
        }
        if outflow_cents < 0 {
            return Err(StoreError::PersistFailed {
                message: format!("outflow_cents must be non-negative, got {outflow_cents}"),
            });
        }
        for txn_id in reconciled_txn_ids {
            if !self.transaction_exists(txn_id) {
                return Err(StoreError::UnknownTransaction {
                    transaction_id: txn_id.clone(),
                });
            }
        }

        let created_at = self.next_timestamp_us();
        let run_id = self.next_reconciliation_run_id();
        let matched_transaction_count = i64::try_from(reconciled_txn_ids.len()).unwrap_or(i64::MAX);
        let statement_line_ids = self.statement_line_ids_for_transaction_ids(reconciled_txn_ids);
        let run = StoredReconciliationRun::new(
            &run_id,
            month_key,
            checking_account,
            opening_balance_cents,
            ledger_delta_cents,
            expected_closing_balance_cents,
            statement_closing_balance_cents,
            variance_cents,
            reconciled,
            matched_postings,
            matched_transaction_count,
            inflow_cents,
            outflow_cents,
            created_at,
        );
        self.reconciliation_runs.insert(run_id.clone(), run.clone());
        self.reconciliation_statement_line_ids
            .insert(run_id, statement_line_ids);
        Ok(run)
    }

    fn write_reconciliation_run_and_month_close(
        &mut self,
        month_key: &str,
        checking_account: &str,
        opening_balance_cents: i64,
        ledger_delta_cents: i64,
        expected_closing_balance_cents: i64,
        statement_closing_balance_cents: i64,
        variance_cents: i64,
        reconciled: bool,
        matched_postings: i64,
        inflow_cents: i64,
        outflow_cents: i64,
        reconciled_txn_ids: &[TransactionId],
        analytics_artifact_id: Option<&str>,
    ) -> Result<(StoredReconciliationRun, StoredMonthClose), StoreError> {
        if month_key.is_empty() {
            return Err(StoreError::PersistFailed {
                message: "month_key must not be empty".to_owned(),
            });
        }
        if checking_account.is_empty() {
            return Err(StoreError::PersistFailed {
                message: "checking_account must not be empty".to_owned(),
            });
        }
        if matched_postings < 0 {
            return Err(StoreError::PersistFailed {
                message: format!("matched_postings must be non-negative, got {matched_postings}"),
            });
        }
        if inflow_cents < 0 {
            return Err(StoreError::PersistFailed {
                message: format!("inflow_cents must be non-negative, got {inflow_cents}"),
            });
        }
        if outflow_cents < 0 {
            return Err(StoreError::PersistFailed {
                message: format!("outflow_cents must be non-negative, got {outflow_cents}"),
            });
        }
        if let Some(artifact_id) = analytics_artifact_id
            && !self.analytics_artifacts.contains_key(artifact_id)
        {
            return Err(StoreError::UnknownArtifact {
                artifact_id: artifact_id.to_owned(),
            });
        }
        let scope_key = (month_key.to_owned(), checking_account.to_owned());
        if let Some(existing_close_id) = self.month_close_by_scope.get(&scope_key) {
            return Err(StoreError::PersistFailed {
                message: format!(
                    "month '{month_key}' for account '{checking_account}' is already closed by '{existing_close_id}'"
                ),
            });
        }
        for txn_id in reconciled_txn_ids {
            if !self.transaction_exists(txn_id) {
                return Err(StoreError::UnknownTransaction {
                    transaction_id: txn_id.clone(),
                });
            }
        }

        let created_at = self.next_timestamp_us();
        let closed_at = self.next_timestamp_us();
        let run_id = self.next_reconciliation_run_id();
        let close_id = self.next_month_close_id();
        let matched_transaction_count = i64::try_from(reconciled_txn_ids.len()).unwrap_or(i64::MAX);
        let statement_line_ids = self.statement_line_ids_for_transaction_ids(reconciled_txn_ids);
        let run = StoredReconciliationRun::new(
            &run_id,
            month_key,
            checking_account,
            opening_balance_cents,
            ledger_delta_cents,
            expected_closing_balance_cents,
            statement_closing_balance_cents,
            variance_cents,
            reconciled,
            matched_postings,
            matched_transaction_count,
            inflow_cents,
            outflow_cents,
            created_at,
        );
        let close = StoredMonthClose::new(
            &close_id,
            month_key,
            checking_account,
            &run_id,
            analytics_artifact_id,
            closed_at,
        );
        self.reconciliation_runs.insert(run_id.clone(), run.clone());
        self.reconciliation_statement_line_ids
            .insert(run_id, statement_line_ids);
        self.month_close_by_scope
            .insert(scope_key, close_id.clone());
        self.month_closes.insert(close_id, close.clone());
        Ok((run, close))
    }

    fn write_month_close(
        &mut self,
        month_key: &str,
        checking_account: &str,
        reconciliation_run_id: &str,
        analytics_artifact_id: Option<&str>,
    ) -> Result<StoredMonthClose, StoreError> {
        if month_key.is_empty() {
            return Err(StoreError::PersistFailed {
                message: "month_key must not be empty".to_owned(),
            });
        }
        if checking_account.is_empty() {
            return Err(StoreError::PersistFailed {
                message: "checking_account must not be empty".to_owned(),
            });
        }
        if reconciliation_run_id.is_empty() {
            return Err(StoreError::PersistFailed {
                message: "reconciliation_run_id must not be empty".to_owned(),
            });
        }

        let run = self
            .reconciliation_runs
            .get(reconciliation_run_id)
            .cloned()
            .ok_or_else(|| StoreError::PersistFailed {
                message: format!("unknown reconciliation run '{reconciliation_run_id}'"),
            })?;
        if run.month_key() != month_key {
            return Err(StoreError::PersistFailed {
                message: format!(
                    "month close month '{month_key}' does not match reconciliation run month '{}'",
                    run.month_key()
                ),
            });
        }
        if run.checking_account() != checking_account {
            return Err(StoreError::PersistFailed {
                message: format!(
                    "month close checking_account '{checking_account}' does not match reconciliation run account '{}'",
                    run.checking_account()
                ),
            });
        }
        if let Some(artifact_id) = analytics_artifact_id
            && !self.analytics_artifacts.contains_key(artifact_id)
        {
            return Err(StoreError::UnknownArtifact {
                artifact_id: artifact_id.to_owned(),
            });
        }
        let scope_key = (month_key.to_owned(), checking_account.to_owned());
        if let Some(existing_close_id) = self.month_close_by_scope.get(&scope_key) {
            return Err(StoreError::PersistFailed {
                message: format!(
                    "month '{month_key}' for account '{checking_account}' is already closed by '{existing_close_id}'"
                ),
            });
        }

        let closed_at = self.next_timestamp_us();
        let close_id = self.next_month_close_id();
        let close = StoredMonthClose::new(
            &close_id,
            month_key,
            checking_account,
            reconciliation_run_id,
            analytics_artifact_id,
            closed_at,
        );
        self.month_close_by_scope
            .insert(scope_key, close_id.clone());
        self.month_closes.insert(close_id, close.clone());
        Ok(close)
    }
}

fn system_time_us() -> Timestamp {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| i64::try_from(duration.as_micros()).unwrap_or(i64::MAX))
        .unwrap_or(0)
}
