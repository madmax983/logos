//! The core abstraction for persistent storage in `logos`.
//!
//! This module defines the [`LedgerStore`] trait, which provides a unified interface
//! for interacting with the financial ledger, regardless of the underlying storage backend
//! (e.g., `PostgreSQL`, in-memory).

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

    /// Projects the ledger's financial truth at a specific historical moment.
    ///
    /// In a bitemporal system, querying the ledger requires two dimensions of time:
    /// 1. `valid_time_us`: When the transaction actually occurred in the real world (e.g., the date on a receipt).
    /// 2. `tx_time_us`: When the system recorded the transaction (e.g., when the import job ran).
    ///
    /// This method allows us to answer questions like: "What did we *think* our balance was at the end of last month,
    /// based on the information we had at that time?"
    ///
    /// ## Examples
    ///
    /// ```
    /// use logos_store::traits::LedgerStore;
    /// use logos_store::memory::MemoryStore;
    ///
    /// let mut store = MemoryStore::new_in_memory();
    /// // Querying at time 0 will yield an empty ledger.
    /// let txns = store.transactions_as_of_us(0, 0).expect("failed to load transactions");
    /// assert!(txns.is_empty());
    /// ```
    ///
    /// # Errors
    ///
    /// Returns a [`StoreError`] if the underlying storage backend fails to execute the query.
    fn transactions_as_of_us(
        &self,
        _valid_time_us: i64,
        _tx_time_us: i64,
    ) -> Result<Vec<StoredTransaction>, StoreError> {
        Err(missing_load("transactions_as_of_us"))
    }

    /// Commits a perfectly balanced transaction into the permanent ledger.
    ///
    /// This is the primary gateway for all new financial data entering the system.
    /// By requiring a [`TransactionBuilder`], we enforce at compile-time that
    /// unbalanced, corrupted entries can never reach the database.
    ///
    /// ## Examples
    ///
    /// ```
    /// use logos_core::domain::transaction::{Posting, TransactionBuilder};
    /// use logos_core::AccountId;
    /// use logos_store::memory::MemoryStore;
    /// use logos_store::traits::LedgerStore;
    ///
    /// let mut store = MemoryStore::new_in_memory();
    /// let builder = TransactionBuilder::new("Groceries")
    ///     .posting(Posting::debit(AccountId::new("expenses:food").unwrap(), 5000).unwrap())
    ///     .posting(Posting::credit(AccountId::new("assets:checking").unwrap(), 5000).unwrap());
    ///
    /// let tx_id = store.write_transaction(builder).expect("failed to persist transaction");
    /// assert!(store.has_transaction(&tx_id));
    /// ```
    ///
    /// # Errors
    ///
    /// Returns a [`StoreError`] if the database transaction fails or disk space is exhausted.
    fn write_transaction(
        &mut self,
        _builder: TransactionBuilder,
    ) -> Result<TransactionId, StoreError> {
        Err(missing_write("write_transaction"))
    }

    /// Commits a balanced transaction with an explicit historical date.
    ///
    /// Unlike `write_transaction` which defaults to the system's current clock,
    /// this method is crucial for data imports where the real-world transaction
    /// happened days or weeks ago (the `valid_from` time).
    ///
    /// ## Examples
    ///
    /// ```
    /// use logos_core::domain::transaction::{Posting, TransactionBuilder};
    /// use logos_core::AccountId;
    /// use logos_store::memory::MemoryStore;
    /// use logos_store::traits::LedgerStore;
    ///
    /// let mut store = MemoryStore::new_in_memory();
    /// let builder = TransactionBuilder::new("Late entry")
    ///     .posting(Posting::debit(AccountId::new("expenses:misc").unwrap(), 1000).unwrap())
    ///     .posting(Posting::credit(AccountId::new("assets:checking").unwrap(), 1000).unwrap());
    ///
    /// // Record the transaction as occurring at Unix timestamp 1672531200 (Jan 1, 2023).
    /// let tx_id = store.write_transaction_with_valid_time(builder, Some(1672531200))
    ///     .expect("failed to persist historical transaction");
    /// ```
    ///
    /// # Errors
    ///
    /// Returns a [`StoreError`] if the database rejects the write.
    fn write_transaction_with_valid_time(
        &mut self,
        _builder: TransactionBuilder,
        _valid_from: Option<i64>,
    ) -> Result<TransactionId, StoreError> {
        Err(missing_write("write_transaction_with_valid_time"))
    }

    /// Applies an immutable correction to a previously recorded transaction.
    ///
    /// Because financial ledgers must be strict, append-only logs for auditability,
    /// we never `UPDATE` or `DELETE` a row. Instead, we write a [`Correction`]
    /// that points to the flawed transaction and negates/replaces its effects.
    ///
    /// # Errors
    ///
    /// Returns a [`StoreError`] if the target transaction being corrected does not exist.
    fn write_correction(&mut self, _correction: Correction) -> Result<(), StoreError> {
        Err(missing_write("write_correction"))
    }

    /// Establishes the spending limit for a specific category within a given month.
    ///
    /// This sets the starting line for envelope budgeting. Any spending against the
    /// `expense_account_prefix` during `month_key` will drain this target.
    ///
    /// ## Examples
    ///
    /// ```
    /// use logos_store::memory::MemoryStore;
    /// use logos_store::traits::LedgerStore;
    ///
    /// let mut store = MemoryStore::new_in_memory();
    /// // Set the grocery budget for March 2026 to $300.00 (30,000 cents).
    /// store.write_budget_target("2026-03", "expenses:food", 30000)
    ///     .expect("failed to persist budget");
    /// ```
    ///
    /// # Errors
    ///
    /// Returns a [`StoreError`] if the month format is invalid or the database connection is lost.
    fn write_budget_target(
        &mut self,
        _month_key: &str,
        _expense_account_prefix: &str,
        _budget_cents: i64,
    ) -> Result<(), StoreError> {
        Err(missing_write("write_budget_target"))
    }

    /// Registers a data warehouse snapshot pointing to a physical file (e.g., Parquet).
    ///
    /// As the system processes events, it periodically drops materialized views into
    /// long-term storage for reporting. This manifest tells the application exactly
    /// where to find the data and guarantees its integrity via `content_hash`.
    ///
    /// # Errors
    ///
    /// Returns a [`StoreError`] if the manifest serialization fails.
    #[allow(clippy::too_many_arguments)] // Too many primitive arguments represents the physical table structure directly.
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

    /// Registers a data warehouse snapshot with an explicitly provided microsecond timestamp.
    ///
    /// # Errors
    ///
    /// Returns a [`StoreError`] if the underlying write to the storage layer fails.
    #[allow(clippy::too_many_arguments)] // Mirrors the physical schema arguments.
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

    /// Persists a batch of transactions parsed from an external source (like a CSV).
    ///
    /// This serves as an idempotency layer. If a user uploads the exact same bank
    /// statement twice, the `duplicate_count` ensures we do not double-count their
    /// groceries.
    ///
    /// # Errors
    ///
    /// Returns a [`StoreError`] if the batch metadata or underlying records fail to serialize.
    #[allow(clippy::too_many_arguments)] // Necessary for passing the flat batch metadata context alongside the records.
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

    /// Logs an attempt to scrape or fetch data from a financial institution.
    ///
    /// Essential for operational observability. If the background scraper breaks,
    /// this table (`fetch_run`) is the first place an operator looks to find the
    /// `error_summary`.
    ///
    /// # Errors
    ///
    /// Returns a [`StoreError`] if the database fails to record the run.
    #[allow(clippy::too_many_arguments)] // Maps directly to the fetch run table columns.
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

    /// Documents the automated process of matching internal ledger records against external bank statements.
    ///
    /// Reconciliation is the heartbeat of a confident accounting system. This method
    /// tracks the variance (the difference between what the bank says we have and what
    /// our ledger thinks we have). A non-zero `variance_cents` is a massive red flag.
    ///
    /// # Errors
    ///
    /// Returns a [`StoreError`] if the run metadata cannot be saved or if `reconciled_txn_ids` contains unknown transactions.
    #[allow(clippy::too_many_arguments)] // Maps directly to the reconciliation run columns to avoid allocating intermediate structs.
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

    /// Atomically reconciles the ledger and immediately locks the month against future edits.
    ///
    /// This is the final step in a month's lifecycle. Once the reconciliation is perfect
    /// (`variance_cents == 0`), we seal the period so historical reports cannot be mutated.
    ///
    /// # Errors
    ///
    /// Returns a [`StoreError`] if the database transaction fails, or if the period is already closed.
    #[allow(clippy::too_many_arguments)] // Maps directly to the reconciliation run columns.
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

    /// Explicitly marks a financial period as finalized and read-only.
    ///
    /// Any subsequent attempt to write or correct a transaction in a closed month
    /// should be rejected by the domain logic.
    ///
    /// # Errors
    ///
    /// Returns a [`StoreError`] if the month is already closed or if the `reconciliation_run_id` is invalid.
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
