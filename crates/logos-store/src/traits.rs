#![allow(missing_docs)]
//! The Ledger Store Contract
//!
//! This module defines the [`LedgerStore`] trait, which acts as the persistence backbone for all `logos` business logic.
//! It abstracts away the underlying storage mechanism (e.g., `PostgreSQL`, in-memory) to allow domain logic to be easily tested.

#![allow(clippy::missing_errors_doc)]
#![allow(clippy::too_many_arguments)]

use logos_core::{Correction, TransactionBuilder, TransactionId};

use crate::error::StoreError;
use crate::model::{
    NewImportRecord, StoredAnalyticsArtifactManifest, StoredBudgetTarget,
    StoredFetchArtifactFormat, StoredFetchRun, StoredFetchRunStatus, StoredImportBatch,
    StoredImportRecord, StoredMonthClose, StoredReconciliationRun, StoredStatementLine,
    StoredTransaction,
};

#[cold]
#[track_caller]
fn missing_read(method: &'static str) -> ! {
    panic!("LedgerStore::{method} is not implemented")
}

fn missing_load(method: &'static str) -> StoreError {
    StoreError::LoadFailed {
        message: format!("LedgerStore::{method} is not implemented"),
    }
}

fn missing_write(method: &'static str) -> StoreError {
    StoreError::PersistFailed {
        message: format!("LedgerStore::{method} is not implemented"),
    }
}

/// Defines the primary contract for persisting and retrieving ledger entities.
///
/// Implementations of this trait must guarantee strict double-entry semantics where applicable
/// and adhere to the append-only nature of the ledger for transactions.
///
/// ## Examples
///
/// ```
/// use logos_store::{LedgerStore, MemoryStore};
///
/// let mut store = MemoryStore::new();
/// assert_eq!(store.transaction_count(), 0);
/// ```
pub trait LedgerStore {
    fn transaction_count(&self) -> usize {
        missing_read("transaction_count")
    }

    fn correction_count(&self) -> usize {
        missing_read("correction_count")
    }

    fn has_transaction(&self, _id: &TransactionId) -> bool {
        missing_read("has_transaction")
    }

    fn latest_correction(&self) -> Option<Correction> {
        missing_read("latest_correction")
    }

    fn transactions(&self) -> Vec<StoredTransaction> {
        missing_read("transactions")
    }

    fn budget_target(
        &self,
        _month_key: &str,
        _expense_account_prefix: &str,
    ) -> Option<StoredBudgetTarget> {
        missing_read("budget_target")
    }

    fn budget_targets(&self) -> Vec<StoredBudgetTarget> {
        missing_read("budget_targets")
    }

    fn analytics_artifact(&self, _artifact_id: &str) -> Option<StoredAnalyticsArtifactManifest> {
        missing_read("analytics_artifact")
    }

    fn analytics_artifacts(&self) -> Vec<StoredAnalyticsArtifactManifest> {
        missing_read("analytics_artifacts")
    }

    fn import_record_count(&self) -> usize {
        missing_read("import_record_count")
    }

    fn has_import_record_content_hash(&self, _content_hash_key: &str) -> bool {
        missing_read("has_import_record_content_hash")
    }

    fn import_records(&self) -> Vec<StoredImportRecord> {
        missing_read("import_records")
    }

    fn import_batches(&self) -> Vec<StoredImportBatch> {
        missing_read("import_batches")
    }

    fn statement_line_count(&self) -> usize {
        missing_read("statement_line_count")
    }

    fn statement_lines(&self) -> Vec<StoredStatementLine> {
        missing_read("statement_lines")
    }

    fn fetch_run_count(&self) -> usize {
        missing_read("fetch_run_count")
    }

    fn fetch_run(&self, _run_id: &str) -> Option<StoredFetchRun> {
        missing_read("fetch_run")
    }

    fn fetch_runs(&self) -> Vec<StoredFetchRun> {
        missing_read("fetch_runs")
    }

    fn statement_lines_for_reconciliation_run(&self, _run_id: &str) -> Vec<StoredStatementLine> {
        missing_read("statement_lines_for_reconciliation_run")
    }

    fn reconciliation_run_count(&self) -> usize {
        missing_read("reconciliation_run_count")
    }

    fn reconciliation_run(&self, _run_id: &str) -> Option<StoredReconciliationRun> {
        missing_read("reconciliation_run")
    }

    fn reconciliation_runs(&self) -> Vec<StoredReconciliationRun> {
        missing_read("reconciliation_runs")
    }

    fn month_close_count(&self) -> usize {
        missing_read("month_close_count")
    }

    fn month_close(&self, _close_id: &str) -> Option<StoredMonthClose> {
        missing_read("month_close")
    }

    fn month_close_for_scope(
        &self,
        _month_key: &str,
        _checking_account: &str,
    ) -> Option<StoredMonthClose> {
        missing_read("month_close_for_scope")
    }

    fn month_closes(&self) -> Vec<StoredMonthClose> {
        missing_read("month_closes")
    }

    fn transactions_as_of_us(
        &self,
        _valid_time_us: i64,
        _tx_time_us: i64,
    ) -> Result<Vec<StoredTransaction>, StoreError> {
        Err(missing_load("transactions_as_of_us"))
    }

    fn write_transaction(
        &mut self,
        _builder: TransactionBuilder,
    ) -> Result<TransactionId, StoreError> {
        Err(missing_write("write_transaction"))
    }

    fn write_transaction_with_valid_time(
        &mut self,
        _builder: TransactionBuilder,
        _valid_from: Option<i64>,
    ) -> Result<TransactionId, StoreError> {
        Err(missing_write("write_transaction_with_valid_time"))
    }

    fn write_correction(&mut self, _correction: Correction) -> Result<(), StoreError> {
        Err(missing_write("write_correction"))
    }

    fn write_budget_target(
        &mut self,
        _month_key: &str,
        _expense_account_prefix: &str,
        _budget_cents: i64,
    ) -> Result<(), StoreError> {
        Err(missing_write("write_budget_target"))
    }

    #[allow(clippy::too_many_arguments)]
    fn write_analytics_artifact_manifest(
        &mut self,
        _artifact_kind: &str,
        _artifact_uri: &str,
        _content_hash: &str,
        _schema_version: i64,
        _row_count: i64,
        _snapshot_valid_at_us: i64,
        _snapshot_tx_at_us: i64,
        _supersedes_artifact_id: Option<&str>,
    ) -> Result<StoredAnalyticsArtifactManifest, StoreError> {
        Err(missing_write("write_analytics_artifact_manifest"))
    }

    #[allow(clippy::too_many_arguments)]
    fn write_analytics_artifact_manifest_us(
        &mut self,
        _artifact_kind: &str,
        _artifact_uri: &str,
        _content_hash: &str,
        _schema_version: i64,
        _row_count: i64,
        _snapshot_valid_at_us: i64,
        _snapshot_tx_at_us: i64,
        _supersedes_artifact_id: Option<&str>,
    ) -> Result<StoredAnalyticsArtifactManifest, StoreError> {
        Err(missing_write("write_analytics_artifact_manifest_us"))
    }

    #[allow(clippy::too_many_arguments)]
    fn write_import_batch(
        &mut self,
        _import_kind: &str,
        _source_uri: &str,
        _batch_key: &str,
        _duplicate_count: i64,
        _dry_run: bool,
        _ocr_enabled: bool,
        _records: &[NewImportRecord],
    ) -> Result<StoredImportBatch, StoreError> {
        Err(missing_write("write_import_batch"))
    }

    #[allow(clippy::too_many_arguments)]
    fn write_fetch_run(
        &mut self,
        _source_id: &str,
        _institution_id: &str,
        _ledger_account: &str,
        _month_key: &str,
        _status: StoredFetchRunStatus,
        _artifact_path: Option<&str>,
        _output_format: Option<StoredFetchArtifactFormat>,
        _opening_balance_cents: Option<i64>,
        _closing_balance_cents: Option<i64>,
        _error_summary: Option<&str>,
    ) -> Result<StoredFetchRun, StoreError> {
        Err(missing_write("write_fetch_run"))
    }

    #[allow(clippy::too_many_arguments)]
    fn write_reconciliation_run(
        &mut self,
        _month_key: &str,
        _checking_account: &str,
        _opening_balance_cents: i64,
        _ledger_delta_cents: i64,
        _expected_closing_balance_cents: i64,
        _statement_closing_balance_cents: i64,
        _variance_cents: i64,
        _reconciled: bool,
        _matched_postings: i64,
        _inflow_cents: i64,
        _outflow_cents: i64,
        _reconciled_txn_ids: &[TransactionId],
    ) -> Result<StoredReconciliationRun, StoreError> {
        Err(missing_write("write_reconciliation_run"))
    }

    #[allow(clippy::too_many_arguments)]
    fn write_reconciliation_run_and_month_close(
        &mut self,
        _month_key: &str,
        _checking_account: &str,
        _opening_balance_cents: i64,
        _ledger_delta_cents: i64,
        _expected_closing_balance_cents: i64,
        _statement_closing_balance_cents: i64,
        _variance_cents: i64,
        _reconciled: bool,
        _matched_postings: i64,
        _inflow_cents: i64,
        _outflow_cents: i64,
        _reconciled_txn_ids: &[TransactionId],
        _analytics_artifact_id: Option<&str>,
    ) -> Result<(StoredReconciliationRun, StoredMonthClose), StoreError> {
        Err(missing_write("write_reconciliation_run_and_month_close"))
    }

    fn write_month_close(
        &mut self,
        _month_key: &str,
        _checking_account: &str,
        _reconciliation_run_id: &str,
        _analytics_artifact_id: Option<&str>,
    ) -> Result<StoredMonthClose, StoreError> {
        Err(missing_write("write_month_close"))
    }
}
