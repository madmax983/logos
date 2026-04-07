use std::cell::{RefCell, RefMut};
use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};

use diesel::Connection;
use diesel::dsl::{exists, select};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::BigInt;
use logos_core::{AccountId, Correction, Posting, TransactionBuilder, TransactionId};
use logos_store::error::StoreError;
use logos_store::model::{
    NewImportRecord, StoredAnalyticsArtifactManifest, StoredBudgetTarget,
    StoredFetchArtifactFormat, StoredFetchRun, StoredFetchRunStatus, StoredImportBatch,
    StoredImportRecord, StoredMonthClose, StoredReconciliationRun, StoredStatementLine,
    StoredTransaction,
};
use logos_store::traits::LedgerStore;

use crate::error::PgStoreError;
use crate::migrate::{pending_migration_names, run_pending_migrations};
use crate::schema::{
    analytics_artifact_manifests, budget_targets, corrections, fetch_runs, import_batches,
    import_records, month_closes, postings, reconciliation_run_statement_lines,
    reconciliation_run_transactions, reconciliation_runs, statement_lines, transactions,
};

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = transactions)]
struct TransactionRow {
    id: String,
    description: String,
    effective_at_us: i64,
    recorded_at_us: i64,
    source_kind: Option<String>,
    external_ref: Option<String>,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = postings)]
struct PostingRow {
    transaction_id: String,
    ordinal: i32,
    account: String,
    amount_cents: i64,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = corrections)]
struct CorrectionRow {
    correction_id: i64,
    supersedes_txn_id: String,
    reason: String,
    recorded_at_us: i64,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = budget_targets)]
struct BudgetTargetRow {
    month_key: String,
    expense_account_prefix: String,
    budget_cents: i64,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = analytics_artifact_manifests)]
struct AnalyticsArtifactRow {
    artifact_id: String,
    artifact_kind: String,
    artifact_uri: String,
    content_hash: String,
    schema_version: i64,
    row_count: i64,
    snapshot_valid_at_us: i64,
    snapshot_tx_at_us: i64,
    created_at_us: i64,
    supersedes_artifact_id: Option<String>,
    snapshot_key: String,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = import_batches)]
struct ImportBatchRow {
    batch_id: String,
    import_kind: String,
    source_uri: String,
    batch_key: String,
    record_count: i64,
    duplicate_count: i64,
    dry_run: bool,
    ocr_enabled: bool,
    imported_at_us: i64,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = import_records)]
struct ImportRecordRow {
    content_hash_key: String,
    batch_id: String,
    imported_txn_id: Option<String>,
    imported_at_us: i64,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = statement_lines)]
struct StatementLineRow {
    line_id: String,
    batch_id: String,
    source_uri: String,
    statement_timestamp: String,
    memo: String,
    amount_cents: i64,
    imported_txn_id: Option<String>,
    imported_at_us: i64,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = fetch_runs)]
struct FetchRunRow {
    run_id: String,
    source_id: String,
    institution_id: String,
    ledger_account: String,
    month_key: String,
    status: String,
    artifact_path: Option<String>,
    output_format: Option<String>,
    opening_balance_cents: Option<i64>,
    closing_balance_cents: Option<i64>,
    error_summary: Option<String>,
    created_at_us: i64,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = reconciliation_runs)]
struct ReconciliationRunRow {
    run_id: String,
    month_key: String,
    checking_account: String,
    opening_balance_cents: i64,
    ledger_delta_cents: i64,
    expected_closing_balance_cents: i64,
    statement_closing_balance_cents: i64,
    variance_cents: i64,
    reconciled: bool,
    matched_postings: i64,
    matched_transaction_count: i64,
    inflow_cents: i64,
    outflow_cents: i64,
    created_at_us: i64,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = month_closes)]
struct MonthCloseRow {
    close_id: String,
    month_key: String,
    checking_account: String,
    reconciliation_run_id: String,
    analytics_artifact_id: Option<String>,
    closed_at_us: i64,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = transactions)]
struct NewTransactionRow<'a> {
    id: &'a str,
    description: &'a str,
    effective_at_us: i64,
    recorded_at_us: i64,
    source_kind: Option<&'a str>,
    external_ref: Option<&'a str>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = postings)]
struct NewPostingRow<'a> {
    transaction_id: &'a str,
    ordinal: i32,
    account: &'a str,
    amount_cents: i64,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = corrections)]
struct NewCorrectionRow<'a> {
    supersedes_txn_id: &'a str,
    reason: &'a str,
    recorded_at_us: i64,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = budget_targets)]
struct NewBudgetTargetRow<'a> {
    month_key: &'a str,
    expense_account_prefix: &'a str,
    budget_cents: i64,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = analytics_artifact_manifests)]
struct NewAnalyticsArtifactRow<'a> {
    artifact_id: &'a str,
    artifact_kind: &'a str,
    artifact_uri: &'a str,
    content_hash: &'a str,
    schema_version: i64,
    row_count: i64,
    snapshot_valid_at_us: i64,
    snapshot_tx_at_us: i64,
    created_at_us: i64,
    supersedes_artifact_id: Option<&'a str>,
    snapshot_key: &'a str,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = import_batches)]
struct NewImportBatchRow<'a> {
    batch_id: &'a str,
    import_kind: &'a str,
    source_uri: &'a str,
    batch_key: &'a str,
    record_count: i64,
    duplicate_count: i64,
    dry_run: bool,
    ocr_enabled: bool,
    imported_at_us: i64,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = import_records)]
struct NewImportRecordRow<'a> {
    content_hash_key: &'a str,
    batch_id: &'a str,
    imported_txn_id: Option<&'a str>,
    imported_at_us: i64,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = statement_lines)]
struct NewStatementLineRow<'a> {
    line_id: &'a str,
    batch_id: &'a str,
    source_uri: &'a str,
    statement_timestamp: &'a str,
    memo: &'a str,
    amount_cents: i64,
    imported_txn_id: Option<&'a str>,
    imported_at_us: i64,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = fetch_runs)]
struct NewFetchRunRow<'a> {
    run_id: &'a str,
    source_id: &'a str,
    institution_id: &'a str,
    ledger_account: &'a str,
    month_key: &'a str,
    status: &'a str,
    artifact_path: Option<&'a str>,
    output_format: Option<&'a str>,
    opening_balance_cents: Option<i64>,
    closing_balance_cents: Option<i64>,
    error_summary: Option<&'a str>,
    created_at_us: i64,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = reconciliation_runs)]
struct NewReconciliationRunRow<'a> {
    run_id: &'a str,
    month_key: &'a str,
    checking_account: &'a str,
    opening_balance_cents: i64,
    ledger_delta_cents: i64,
    expected_closing_balance_cents: i64,
    statement_closing_balance_cents: i64,
    variance_cents: i64,
    reconciled: bool,
    matched_postings: i64,
    matched_transaction_count: i64,
    inflow_cents: i64,
    outflow_cents: i64,
    created_at_us: i64,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = reconciliation_run_transactions)]
struct NewReconciliationRunTransactionRow<'a> {
    run_id: &'a str,
    transaction_id: &'a str,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = reconciliation_run_statement_lines)]
struct NewReconciliationRunStatementLineRow<'a> {
    run_id: &'a str,
    statement_line_id: &'a str,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = month_closes)]
struct NewMonthCloseRow<'a> {
    close_id: &'a str,
    month_key: &'a str,
    checking_account: &'a str,
    reconciliation_run_id: &'a str,
    analytics_artifact_id: Option<&'a str>,
    closed_at_us: i64,
}

#[derive(Debug, QueryableByName)]
struct SequenceValueRow {
    #[diesel(sql_type = BigInt)]
    sequence_value: i64,
}

pub struct PostgresStore {
    connection: RefCell<PgConnection>,
}

impl PostgresStore {
    #[allow(clippy::too_many_arguments)]
    fn map_statement_line<'a>(
        line_id: &'a str,
        source_uri: &'a str,
        statement_timestamp: &'a str,
        memo: &'a str,
        amount_cents: i64,
        imported_txn_id: Option<&'a TransactionId>,
        batch_id: &'a str,
        imported_at_us: i64,
    ) -> NewStatementLineRow<'a> {
        NewStatementLineRow {
            line_id,
            batch_id,
            source_uri,
            statement_timestamp,
            memo,
            amount_cents,
            imported_txn_id: imported_txn_id.map(TransactionId::as_str),
            imported_at_us,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn map_stored_reconciliation_and_close(
        run_id: &str,
        close_id: &str,
        month_key: &str,
        checking_account: &str,
        opening_balance_cents: i64,
        ledger_delta_cents: i64,
        expected_closing_balance_cents: i64,
        statement_closing_balance_cents: i64,
        variance_cents: i64,
        reconciled: bool,
        matched_postings: i64,
        matched_transaction_count: i64,
        inflow_cents: i64,
        outflow_cents: i64,
        created_at_us: i64,
        closed_at_us: i64,
        analytics_artifact_id: Option<&str>,
    ) -> (StoredReconciliationRun, StoredMonthClose) {
        (
            StoredReconciliationRun::new(
                run_id,
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
                created_at_us,
            ),
            StoredMonthClose::new(
                close_id,
                month_key,
                checking_account,
                run_id,
                analytics_artifact_id,
                closed_at_us,
            ),
        )
    }

    /// # Errors
    /// Returns `PgStoreError` if connection fails.
    pub fn connect(database_url: &str) -> Result<Self, PgStoreError> {
        if database_url.trim().is_empty() {
            return Err(PgStoreError::MissingDatabaseUrl);
        }

        let connection =
            PgConnection::establish(database_url).map_err(|err| PgStoreError::Connection {
                message: err.to_string(),
            })?;

        Ok(Self {
            connection: RefCell::new(connection),
        })
    }

    #[must_use]
    /// # Errors
    /// Returns `PgStoreError` if connection fails.
    pub fn connection_mut(&self) -> RefMut<'_, PgConnection> {
        self.connection.borrow_mut()
    }

    /// # Errors
    /// Returns `PgStoreError` on fetch failure.
    pub fn pending_migrations(&mut self) -> Result<Vec<String>, PgStoreError> {
        let mut connection = self.connection.borrow_mut();
        pending_migration_names(&mut connection)
    }

    /// # Errors
    /// Returns `PgStoreError` on execution failure.
    pub fn run_migrations(&mut self) -> Result<Vec<String>, PgStoreError> {
        let mut connection = self.connection.borrow_mut();
        run_pending_migrations(&mut connection)
    }

    fn try_transaction_count(&self) -> Result<usize, StoreError> {
        let mut connection = self.connection.borrow_mut();
        let count = transactions::table
            .count()
            .get_result::<i64>(&mut *connection)
            .map_err(|err| load_failure(format!("counting transactions failed: {err}")))?;
        usize::try_from(count)
            .map_err(|_| load_failure(format!("transaction count '{count}' exceeds usize")))
    }

    fn try_correction_count(&self) -> Result<usize, StoreError> {
        let mut connection = self.connection.borrow_mut();
        let count = corrections::table
            .count()
            .get_result::<i64>(&mut *connection)
            .map_err(|err| load_failure(format!("counting corrections failed: {err}")))?;
        usize::try_from(count)
            .map_err(|_| load_failure(format!("correction count '{count}' exceeds usize")))
    }

    fn try_has_transaction(&self, id: &TransactionId) -> Result<bool, StoreError> {
        let mut connection = self.connection.borrow_mut();
        select(exists(
            transactions::table.filter(transactions::id.eq(id.as_str())),
        ))
        .get_result::<bool>(&mut *connection)
        .map_err(|err| {
            load_failure(format!(
                "checking transaction '{}' existence failed: {err}",
                id.as_str()
            ))
        })
    }

    fn try_latest_correction(&self) -> Result<Option<Correction>, StoreError> {
        let mut connection = self.connection.borrow_mut();
        let row = corrections::table
            .order(corrections::correction_id.desc())
            .select(CorrectionRow::as_select())
            .first::<CorrectionRow>(&mut *connection)
            .optional()
            .map_err(|err| load_failure(format!("loading latest correction failed: {err}")))?;
        row.map(|r| Self::correction_from_row(&r)).transpose()
    }

    fn try_transactions(&self) -> Result<Vec<StoredTransaction>, StoreError> {
        let mut connection = self.connection.borrow_mut();
        let rows = transactions::table
            .order(transactions::id.asc())
            .select(TransactionRow::as_select())
            .load::<TransactionRow>(&mut *connection)
            .map_err(|err| load_failure(format!("loading transactions failed: {err}")))?;
        Self::hydrate_transactions(&mut connection, rows)
    }

    fn next_transaction_id(connection: &mut PgConnection) -> Result<TransactionId, StoreError> {
        let sequence = sql_query("SELECT nextval('transaction_id_seq') AS sequence_value")
            .get_result::<SequenceValueRow>(connection)
            .map_err(|err| {
                persist_failure(format!(
                    "allocating next transaction id sequence failed: {err}"
                ))
            })?;
        TransactionId::new(&format!("txn-{}", sequence.sequence_value)).map_err(StoreError::from)
    }

    fn hydrate_transactions(
        connection: &mut PgConnection,
        rows: Vec<TransactionRow>,
    ) -> Result<Vec<StoredTransaction>, StoreError> {
        if rows.is_empty() {
            return Ok(Vec::new());
        }

        let transaction_ids: Vec<&str> = rows.iter().map(|row| row.id.as_str()).collect();
        let posting_rows = postings::table
            .filter(postings::transaction_id.eq_any(&transaction_ids))
            .order((postings::transaction_id.asc(), postings::ordinal.asc()))
            .select(PostingRow::as_select())
            .load::<PostingRow>(connection)
            .map_err(|err| load_failure(format!("loading postings failed: {err}")))?;

        let mut postings_by_transaction: HashMap<String, Vec<PostingRow>> =
            HashMap::with_capacity(transaction_ids.len());
        for posting_row in posting_rows {
            postings_by_transaction
                .entry(posting_row.transaction_id.clone())
                .or_default()
                .push(posting_row);
        }

        rows.into_iter()
            .map(|row| {
                let posting_rows = postings_by_transaction.remove(&row.id).ok_or_else(|| {
                    load_failure(format!("transaction '{}' has no postings", row.id))
                })?;
                Self::stored_transaction_from_rows(&row, posting_rows)
            })
            .collect()
    }

    fn stored_transaction_from_rows(
        row: &TransactionRow,
        posting_rows: Vec<PostingRow>,
    ) -> Result<StoredTransaction, StoreError> {
        let txn_id = TransactionId::new(&row.id).map_err(|err| {
            load_failure(format!("invalid stored transaction id '{}': {err}", row.id))
        })?;
        let mut builder = TransactionBuilder::new(&row.description);
        for posting_row in posting_rows {
            builder = builder.posting(Self::posting_from_row(&txn_id, &posting_row)?);
        }
        let transaction = builder.build().map_err(|err| {
            load_failure(format!(
                "stored transaction '{}' failed validation: {err}",
                txn_id.as_str()
            ))
        })?;

        let _ = row.recorded_at_us;
        let _ = row.source_kind;
        let _ = row.external_ref;

        Ok(StoredTransaction::with_effective_at(
            txn_id,
            transaction,
            row.effective_at_us,
        ))
    }

    fn posting_from_row(
        transaction_id: &TransactionId,
        row: &PostingRow,
    ) -> Result<Posting, StoreError> {
        let account = AccountId::new(&row.account).map_err(|err| {
            load_failure(format!(
                "stored transaction '{}' has invalid account '{}' at ordinal {}: {err}",
                transaction_id.as_str(),
                row.account,
                row.ordinal
            ))
        })?;

        match row.amount_cents.cmp(&0) {
            std::cmp::Ordering::Greater => {
                Posting::debit(account, row.amount_cents).map_err(|err| {
                    load_failure(format!(
                        "stored transaction '{}' has invalid debit at ordinal {}: {err}",
                        transaction_id.as_str(),
                        row.ordinal
                    ))
                })
            }
            std::cmp::Ordering::Less => {
                let credit_amount = row.amount_cents.checked_abs().ok_or_else(|| {
                    load_failure(format!(
                        "stored transaction '{}' has overflowing credit at ordinal {}",
                        transaction_id.as_str(),
                        row.ordinal
                    ))
                })?;
                Posting::credit(account, credit_amount).map_err(|err| {
                    load_failure(format!(
                        "stored transaction '{}' has invalid credit at ordinal {}: {err}",
                        transaction_id.as_str(),
                        row.ordinal
                    ))
                })
            }
            std::cmp::Ordering::Equal => Err(load_failure(format!(
                "stored transaction '{}' has zero-amount posting at ordinal {}",
                transaction_id.as_str(),
                row.ordinal
            ))),
        }
    }

    fn correction_from_row(row: &CorrectionRow) -> Result<Correction, StoreError> {
        let supersedes_id = TransactionId::new(&row.supersedes_txn_id).map_err(|err| {
            load_failure(format!(
                "invalid stored correction target '{}': {err}",
                row.supersedes_txn_id
            ))
        })?;
        let _ = row.correction_id;
        let _ = row.recorded_at_us;
        Correction::new(supersedes_id, &row.reason).map_err(|err| {
            load_failure(format!(
                "invalid stored correction for '{}': {err}",
                row.supersedes_txn_id
            ))
        })
    }

    fn parse_transaction_id(value: &str, context: &str) -> Result<TransactionId, StoreError> {
        TransactionId::new(value).map_err(|err| {
            load_failure(format!(
                "invalid transaction id '{value}' for {context}: {err}"
            ))
        })
    }

    fn budget_target_from_row(row: &BudgetTargetRow) -> StoredBudgetTarget {
        StoredBudgetTarget::new(
            &row.month_key,
            &row.expense_account_prefix,
            row.budget_cents,
        )
    }

    fn analytics_artifact_from_row(row: &AnalyticsArtifactRow) -> StoredAnalyticsArtifactManifest {
        StoredAnalyticsArtifactManifest::new(
            &row.artifact_id,
            &row.artifact_kind,
            &row.artifact_uri,
            &row.content_hash,
            row.schema_version,
            row.row_count,
            row.snapshot_valid_at_us,
            row.snapshot_tx_at_us,
            row.created_at_us,
            row.supersedes_artifact_id.as_deref(),
            &row.snapshot_key,
        )
    }

    fn import_batch_from_row(row: &ImportBatchRow) -> StoredImportBatch {
        StoredImportBatch::new(
            &row.batch_id,
            &row.import_kind,
            &row.source_uri,
            &row.batch_key,
            row.record_count,
            row.duplicate_count,
            row.dry_run,
            row.ocr_enabled,
            row.imported_at_us,
        )
    }

    fn import_record_from_row(row: &ImportRecordRow) -> Result<StoredImportRecord, StoreError> {
        let imported_txn_id = row
            .imported_txn_id
            .as_deref()
            .map(|value| Self::parse_transaction_id(value, "import record"))
            .transpose()?;
        Ok(StoredImportRecord::new(
            &row.content_hash_key,
            &row.batch_id,
            imported_txn_id,
            row.imported_at_us,
        ))
    }

    fn statement_line_from_row(row: &StatementLineRow) -> Result<StoredStatementLine, StoreError> {
        let imported_txn_id = row
            .imported_txn_id
            .as_deref()
            .map(|value| Self::parse_transaction_id(value, "statement line"))
            .transpose()?;
        Ok(StoredStatementLine::new(
            &row.line_id,
            &row.batch_id,
            &row.source_uri,
            &row.statement_timestamp,
            &row.memo,
            row.amount_cents,
            imported_txn_id,
            row.imported_at_us,
        ))
    }

    fn fetch_run_from_row(row: &FetchRunRow) -> Result<StoredFetchRun, StoreError> {
        let status = StoredFetchRunStatus::parse(&row.status).ok_or_else(|| {
            load_failure(format!(
                "invalid fetch run status '{}' for '{}'",
                row.status, row.run_id
            ))
        })?;
        let output_format = row
            .output_format
            .as_deref()
            .map(|value| {
                StoredFetchArtifactFormat::parse(value).ok_or_else(|| {
                    load_failure(format!(
                        "invalid fetch run output_format '{}' for '{}'",
                        value, row.run_id
                    ))
                })
            })
            .transpose()?;
        Ok(StoredFetchRun::new(
            &row.run_id,
            &row.source_id,
            &row.institution_id,
            &row.ledger_account,
            &row.month_key,
            status,
            row.artifact_path.as_deref(),
            output_format,
            row.opening_balance_cents,
            row.closing_balance_cents,
            row.error_summary.as_deref(),
            row.created_at_us,
        ))
    }

    fn reconciliation_run_from_row(row: &ReconciliationRunRow) -> StoredReconciliationRun {
        StoredReconciliationRun::new(
            &row.run_id,
            &row.month_key,
            &row.checking_account,
            row.opening_balance_cents,
            row.ledger_delta_cents,
            row.expected_closing_balance_cents,
            row.statement_closing_balance_cents,
            row.variance_cents,
            row.reconciled,
            row.matched_postings,
            row.matched_transaction_count,
            row.inflow_cents,
            row.outflow_cents,
            row.created_at_us,
        )
    }

    fn month_close_from_row(row: &MonthCloseRow) -> StoredMonthClose {
        StoredMonthClose::new(
            &row.close_id,
            &row.month_key,
            &row.checking_account,
            &row.reconciliation_run_id,
            row.analytics_artifact_id.as_deref(),
            row.closed_at_us,
        )
    }

    fn next_named_id(
        connection: &mut PgConnection,
        sequence_name: &str,
        prefix: &str,
    ) -> Result<String, StoreError> {
        let query = format!("SELECT nextval('{sequence_name}') AS sequence_value");
        let sequence = sql_query(query)
            .get_result::<SequenceValueRow>(connection)
            .map_err(|err| {
                persist_failure(format!(
                    "allocating next '{prefix}' id from sequence '{sequence_name}' failed: {err}"
                ))
            })?;
        Ok(format!("{prefix}{}", sequence.sequence_value))
    }

    fn next_artifact_id(connection: &mut PgConnection) -> Result<String, StoreError> {
        Self::next_named_id(connection, "artifact_id_seq", "artifact-")
    }

    fn next_import_batch_id(connection: &mut PgConnection) -> Result<String, StoreError> {
        Self::next_named_id(connection, "import_batch_id_seq", "import-batch-")
    }

    fn next_statement_line_id(connection: &mut PgConnection) -> Result<String, StoreError> {
        Self::next_named_id(connection, "statement_line_id_seq", "stmt-line-")
    }

    fn next_fetch_run_id(connection: &mut PgConnection) -> Result<String, StoreError> {
        Self::next_named_id(connection, "fetch_run_id_seq", "fetch-")
    }

    fn next_reconciliation_run_id(connection: &mut PgConnection) -> Result<String, StoreError> {
        Self::next_named_id(connection, "reconciliation_run_id_seq", "recon-")
    }

    fn next_month_close_id(connection: &mut PgConnection) -> Result<String, StoreError> {
        Self::next_named_id(connection, "month_close_id_seq", "close-")
    }

    fn try_budget_target(
        &self,
        month_key: &str,
        expense_account_prefix: &str,
    ) -> Result<Option<StoredBudgetTarget>, StoreError> {
        let mut connection = self.connection.borrow_mut();
        let row = budget_targets::table
            .filter(budget_targets::month_key.eq(month_key))
            .filter(budget_targets::expense_account_prefix.eq(expense_account_prefix))
            .select(BudgetTargetRow::as_select())
            .first::<BudgetTargetRow>(&mut *connection)
            .optional()
            .map_err(|err| load_failure(format!("loading budget target failed: {err}")))?;
        Ok(row.as_ref().map(Self::budget_target_from_row))
    }

    fn try_budget_targets(&self) -> Result<Vec<StoredBudgetTarget>, StoreError> {
        let mut connection = self.connection.borrow_mut();
        budget_targets::table
            .order((
                budget_targets::month_key.asc(),
                budget_targets::expense_account_prefix.asc(),
            ))
            .select(BudgetTargetRow::as_select())
            .load::<BudgetTargetRow>(&mut *connection)
            .map(|rows| {
                rows.into_iter()
                    .map(|r| Self::budget_target_from_row(&r))
                    .collect()
            })
            .map_err(|err| load_failure(format!("loading budget targets failed: {err}")))
    }

    fn try_analytics_artifact(
        &self,
        artifact_id: &str,
    ) -> Result<Option<StoredAnalyticsArtifactManifest>, StoreError> {
        let mut connection = self.connection.borrow_mut();
        let row = analytics_artifact_manifests::table
            .filter(analytics_artifact_manifests::artifact_id.eq(artifact_id))
            .select(AnalyticsArtifactRow::as_select())
            .first::<AnalyticsArtifactRow>(&mut *connection)
            .optional()
            .map_err(|err| load_failure(format!("loading analytics artifact failed: {err}")))?;
        Ok(row.map(|r| Self::analytics_artifact_from_row(&r)))
    }

    fn try_analytics_artifacts(&self) -> Result<Vec<StoredAnalyticsArtifactManifest>, StoreError> {
        let mut connection = self.connection.borrow_mut();
        analytics_artifact_manifests::table
            .order(analytics_artifact_manifests::artifact_id.asc())
            .select(AnalyticsArtifactRow::as_select())
            .load::<AnalyticsArtifactRow>(&mut *connection)
            .map(|rows| {
                rows.into_iter()
                    .map(|r| Self::analytics_artifact_from_row(&r))
                    .collect()
            })
            .map_err(|err| load_failure(format!("loading analytics artifacts failed: {err}")))
    }

    fn try_import_record_count(&self) -> Result<usize, StoreError> {
        let mut connection = self.connection.borrow_mut();
        let count = import_records::table
            .count()
            .get_result::<i64>(&mut *connection)
            .map_err(|err| load_failure(format!("counting import records failed: {err}")))?;
        usize::try_from(count)
            .map_err(|_| load_failure(format!("import record count '{count}' exceeds usize")))
    }

    fn try_has_import_record_content_hash(
        &self,
        content_hash_key: &str,
    ) -> Result<bool, StoreError> {
        let mut connection = self.connection.borrow_mut();
        select(exists(
            import_records::table.filter(import_records::content_hash_key.eq(content_hash_key)),
        ))
        .get_result::<bool>(&mut *connection)
        .map_err(|err| {
            load_failure(format!(
                "checking import record '{content_hash_key}' existence failed: {err}"
            ))
        })
    }

    fn try_import_records(&self) -> Result<Vec<StoredImportRecord>, StoreError> {
        let mut connection = self.connection.borrow_mut();
        let rows = import_records::table
            .order(import_records::content_hash_key.asc())
            .select(ImportRecordRow::as_select())
            .load::<ImportRecordRow>(&mut *connection)
            .map_err(|err| load_failure(format!("loading import records failed: {err}")))?;
        rows.into_iter()
            .map(|r| Self::import_record_from_row(&r))
            .collect()
    }

    fn try_import_batches(&self) -> Result<Vec<StoredImportBatch>, StoreError> {
        let mut connection = self.connection.borrow_mut();
        import_batches::table
            .order(import_batches::batch_id.asc())
            .select(ImportBatchRow::as_select())
            .load::<ImportBatchRow>(&mut *connection)
            .map(|rows| {
                rows.into_iter()
                    .map(|r| Self::import_batch_from_row(&r))
                    .collect()
            })
            .map_err(|err| load_failure(format!("loading import batches failed: {err}")))
    }

    fn try_statement_line_count(&self) -> Result<usize, StoreError> {
        let mut connection = self.connection.borrow_mut();
        let count = statement_lines::table
            .count()
            .get_result::<i64>(&mut *connection)
            .map_err(|err| load_failure(format!("counting statement lines failed: {err}")))?;
        usize::try_from(count)
            .map_err(|_| load_failure(format!("statement line count '{count}' exceeds usize")))
    }

    fn try_statement_lines(&self) -> Result<Vec<StoredStatementLine>, StoreError> {
        let mut connection = self.connection.borrow_mut();
        let rows = statement_lines::table
            .order(statement_lines::line_id.asc())
            .select(StatementLineRow::as_select())
            .load::<StatementLineRow>(&mut *connection)
            .map_err(|err| load_failure(format!("loading statement lines failed: {err}")))?;
        rows.into_iter()
            .map(|r| Self::statement_line_from_row(&r))
            .collect()
    }

    fn try_fetch_run_count(&self) -> Result<usize, StoreError> {
        let mut connection = self.connection.borrow_mut();
        let count = fetch_runs::table
            .count()
            .get_result::<i64>(&mut *connection)
            .map_err(|err| load_failure(format!("counting fetch runs failed: {err}")))?;
        usize::try_from(count)
            .map_err(|_| load_failure(format!("fetch run count '{count}' exceeds usize")))
    }

    fn try_fetch_run(&self, run_id: &str) -> Result<Option<StoredFetchRun>, StoreError> {
        let mut connection = self.connection.borrow_mut();
        let row = fetch_runs::table
            .filter(fetch_runs::run_id.eq(run_id))
            .select(FetchRunRow::as_select())
            .first::<FetchRunRow>(&mut *connection)
            .optional()
            .map_err(|err| load_failure(format!("loading fetch run failed: {err}")))?;
        row.map(|r| Self::fetch_run_from_row(&r)).transpose()
    }

    fn try_fetch_runs(&self) -> Result<Vec<StoredFetchRun>, StoreError> {
        let mut connection = self.connection.borrow_mut();
        let rows = fetch_runs::table
            .order(fetch_runs::run_id.asc())
            .select(FetchRunRow::as_select())
            .load::<FetchRunRow>(&mut *connection)
            .map_err(|err| load_failure(format!("loading fetch runs failed: {err}")))?;
        rows.into_iter()
            .map(|r| Self::fetch_run_from_row(&r))
            .collect()
    }

    fn try_statement_lines_for_reconciliation_run(
        &self,
        run_id: &str,
    ) -> Result<Vec<StoredStatementLine>, StoreError> {
        let mut connection = self.connection.borrow_mut();
        let line_ids = reconciliation_run_statement_lines::table
            .filter(reconciliation_run_statement_lines::run_id.eq(run_id))
            .select(reconciliation_run_statement_lines::statement_line_id)
            .order(reconciliation_run_statement_lines::statement_line_id.asc())
            .load::<String>(&mut *connection)
            .map_err(|err| {
                load_failure(format!(
                    "loading reconciliation statement line ids for '{run_id}' failed: {err}"
                ))
            })?;
        if line_ids.is_empty() {
            return Ok(Vec::new());
        }
        let rows = statement_lines::table
            .filter(statement_lines::line_id.eq_any(&line_ids))
            .order(statement_lines::line_id.asc())
            .select(StatementLineRow::as_select())
            .load::<StatementLineRow>(&mut *connection)
            .map_err(|err| {
                load_failure(format!(
                    "loading statement lines for reconciliation run '{run_id}' failed: {err}"
                ))
            })?;
        rows.into_iter()
            .map(|r| Self::statement_line_from_row(&r))
            .collect()
    }

    fn try_reconciliation_run_count(&self) -> Result<usize, StoreError> {
        let mut connection = self.connection.borrow_mut();
        let count = reconciliation_runs::table
            .count()
            .get_result::<i64>(&mut *connection)
            .map_err(|err| load_failure(format!("counting reconciliation runs failed: {err}")))?;
        usize::try_from(count)
            .map_err(|_| load_failure(format!("reconciliation run count '{count}' exceeds usize")))
    }

    fn try_reconciliation_run(
        &self,
        run_id: &str,
    ) -> Result<Option<StoredReconciliationRun>, StoreError> {
        let mut connection = self.connection.borrow_mut();
        let row = reconciliation_runs::table
            .filter(reconciliation_runs::run_id.eq(run_id))
            .select(ReconciliationRunRow::as_select())
            .first::<ReconciliationRunRow>(&mut *connection)
            .optional()
            .map_err(|err| load_failure(format!("loading reconciliation run failed: {err}")))?;
        Ok(row.as_ref().map(Self::reconciliation_run_from_row))
    }

    fn try_reconciliation_runs(&self) -> Result<Vec<StoredReconciliationRun>, StoreError> {
        let mut connection = self.connection.borrow_mut();
        reconciliation_runs::table
            .order(reconciliation_runs::run_id.asc())
            .select(ReconciliationRunRow::as_select())
            .load::<ReconciliationRunRow>(&mut *connection)
            .map(|rows| rows.iter().map(Self::reconciliation_run_from_row).collect())
            .map_err(|err| load_failure(format!("loading reconciliation runs failed: {err}")))
    }

    fn try_month_close_count(&self) -> Result<usize, StoreError> {
        let mut connection = self.connection.borrow_mut();
        let count = month_closes::table
            .count()
            .get_result::<i64>(&mut *connection)
            .map_err(|err| load_failure(format!("counting month closes failed: {err}")))?;
        usize::try_from(count)
            .map_err(|_| load_failure(format!("month close count '{count}' exceeds usize")))
    }

    fn try_month_close(&self, close_id: &str) -> Result<Option<StoredMonthClose>, StoreError> {
        let mut connection = self.connection.borrow_mut();
        let row = month_closes::table
            .filter(month_closes::close_id.eq(close_id))
            .select(MonthCloseRow::as_select())
            .first::<MonthCloseRow>(&mut *connection)
            .optional()
            .map_err(|err| load_failure(format!("loading month close failed: {err}")))?;
        Ok(row.as_ref().map(Self::month_close_from_row))
    }

    fn try_month_close_for_scope(
        &self,
        month_key: &str,
        checking_account: &str,
    ) -> Result<Option<StoredMonthClose>, StoreError> {
        let mut connection = self.connection.borrow_mut();
        let row = month_closes::table
            .filter(month_closes::month_key.eq(month_key))
            .filter(month_closes::checking_account.eq(checking_account))
            .select(MonthCloseRow::as_select())
            .first::<MonthCloseRow>(&mut *connection)
            .optional()
            .map_err(|err| load_failure(format!("loading month close for scope failed: {err}")))?;
        Ok(row.as_ref().map(Self::month_close_from_row))
    }

    fn try_month_closes(&self) -> Result<Vec<StoredMonthClose>, StoreError> {
        let mut connection = self.connection.borrow_mut();
        month_closes::table
            .order(month_closes::close_id.asc())
            .select(MonthCloseRow::as_select())
            .load::<MonthCloseRow>(&mut *connection)
            .map(|rows| rows.iter().map(Self::month_close_from_row).collect())
            .map_err(|err| load_failure(format!("loading month closes failed: {err}")))
    }

    fn ensure_transactions_exist(
        connection: &mut PgConnection,
        transaction_ids: &[TransactionId],
    ) -> Result<(), StoreError> {
        for transaction_id in transaction_ids {
            let exists = select(exists(
                transactions::table.filter(transactions::id.eq(transaction_id.as_str())),
            ))
            .get_result::<bool>(connection)
            .map_err(|err| {
                persist_failure(format!(
                    "checking transaction '{}' existence failed: {err}",
                    transaction_id.as_str()
                ))
            })?;
            if !exists {
                return Err(StoreError::UnknownTransaction {
                    transaction_id: transaction_id.clone(),
                });
            }
        }
        Ok(())
    }

    fn statement_line_ids_for_transaction_ids(
        connection: &mut PgConnection,
        transaction_ids: &[TransactionId],
    ) -> Result<Vec<String>, StoreError> {
        if transaction_ids.is_empty() {
            return Ok(Vec::new());
        }
        let ids: Vec<&str> = transaction_ids.iter().map(TransactionId::as_str).collect();
        statement_lines::table
            .filter(statement_lines::imported_txn_id.eq_any(ids))
            .select(statement_lines::line_id)
            .order(statement_lines::line_id.asc())
            .load::<String>(connection)
            .map_err(|err| {
                persist_failure(format!(
                    "loading statement lines for reconciled transactions failed: {err}"
                ))
            })
    }

    fn try_validate_reconciliation_run_params(
        month_key: &str,
        checking_account: &str,
        matched_postings: i64,
        inflow_cents: i64,
        outflow_cents: i64,
    ) -> Result<(), StoreError> {
        if month_key.is_empty() {
            return Err(persist_failure("month_key must not be empty".to_owned()));
        }
        if checking_account.is_empty() {
            return Err(persist_failure(
                "checking_account must not be empty".to_owned(),
            ));
        }
        if matched_postings < 0 {
            return Err(persist_failure(format!(
                "matched_postings must be non-negative, got {matched_postings}"
            )));
        }
        if inflow_cents < 0 {
            return Err(persist_failure(format!(
                "inflow_cents must be non-negative, got {inflow_cents}"
            )));
        }
        if outflow_cents < 0 {
            return Err(persist_failure(format!(
                "outflow_cents must be non-negative, got {outflow_cents}"
            )));
        }
        Ok(())
    }

    fn try_validate_import_records(
        &self,
        import_kind: &str,
        source_uri: &str,
        batch_key: &str,
        duplicate_count: i64,
        records: &[NewImportRecord],
    ) -> Result<(), StoreError> {
        if import_kind.is_empty() {
            return Err(persist_failure("import_kind must not be empty".to_owned()));
        }
        if source_uri.is_empty() {
            return Err(persist_failure("source_uri must not be empty".to_owned()));
        }
        if batch_key.is_empty() {
            return Err(persist_failure("batch_key must not be empty".to_owned()));
        }
        if duplicate_count < 0 {
            return Err(persist_failure(format!(
                "duplicate_count must be non-negative, got {duplicate_count}"
            )));
        }

        let mut seen_keys = HashSet::new();
        for record in records {
            let key = record.content_hash_key();
            if key.is_empty() {
                return Err(persist_failure(
                    "import record content_hash_key must not be empty".to_owned(),
                ));
            }
            if !seen_keys.insert(key.to_owned()) {
                return Err(persist_failure(format!(
                    "duplicate import record content_hash_key '{key}' in write payload"
                )));
            }
            if self.try_has_import_record_content_hash(key)? {
                return Err(persist_failure(format!(
                    "import record content_hash_key '{key}' already exists"
                )));
            }
            if let Some(line) = record.statement_line() {
                if line.source_uri().is_empty() {
                    return Err(persist_failure(
                        "statement line source_uri must not be empty".to_owned(),
                    ));
                }
                if line.statement_timestamp().is_empty() {
                    return Err(persist_failure(
                        "statement line timestamp must not be empty".to_owned(),
                    ));
                }
                if line.memo().is_empty() {
                    return Err(persist_failure(
                        "statement line memo must not be empty".to_owned(),
                    ));
                }
            }
        }
        Ok(())
    }
}

impl LedgerStore for PostgresStore {
    fn transaction_count(&self) -> usize {
        expect_read("transaction_count", self.try_transaction_count())
    }

    fn correction_count(&self) -> usize {
        expect_read("correction_count", self.try_correction_count())
    }

    fn has_transaction(&self, id: &TransactionId) -> bool {
        expect_read("has_transaction", self.try_has_transaction(id))
    }

    fn latest_correction(&self) -> Option<Correction> {
        expect_read("latest_correction", self.try_latest_correction())
    }

    fn transactions(&self) -> Vec<StoredTransaction> {
        expect_read("transactions", self.try_transactions())
    }

    fn budget_target(
        &self,
        month_key: &str,
        expense_account_prefix: &str,
    ) -> Option<StoredBudgetTarget> {
        expect_read(
            "budget_target",
            self.try_budget_target(month_key, expense_account_prefix),
        )
    }

    fn budget_targets(&self) -> Vec<StoredBudgetTarget> {
        expect_read("budget_targets", self.try_budget_targets())
    }

    fn analytics_artifact(&self, artifact_id: &str) -> Option<StoredAnalyticsArtifactManifest> {
        expect_read(
            "analytics_artifact",
            self.try_analytics_artifact(artifact_id),
        )
    }

    fn analytics_artifacts(&self) -> Vec<StoredAnalyticsArtifactManifest> {
        expect_read("analytics_artifacts", self.try_analytics_artifacts())
    }

    fn import_record_count(&self) -> usize {
        expect_read("import_record_count", self.try_import_record_count())
    }

    fn has_import_record_content_hash(&self, content_hash_key: &str) -> bool {
        expect_read(
            "has_import_record_content_hash",
            self.try_has_import_record_content_hash(content_hash_key),
        )
    }

    fn import_records(&self) -> Vec<StoredImportRecord> {
        expect_read("import_records", self.try_import_records())
    }

    fn import_batches(&self) -> Vec<StoredImportBatch> {
        expect_read("import_batches", self.try_import_batches())
    }

    fn statement_line_count(&self) -> usize {
        expect_read("statement_line_count", self.try_statement_line_count())
    }

    fn statement_lines(&self) -> Vec<StoredStatementLine> {
        expect_read("statement_lines", self.try_statement_lines())
    }

    fn fetch_run_count(&self) -> usize {
        expect_read("fetch_run_count", self.try_fetch_run_count())
    }

    fn fetch_run(&self, run_id: &str) -> Option<StoredFetchRun> {
        expect_read("fetch_run", self.try_fetch_run(run_id))
    }

    fn fetch_runs(&self) -> Vec<StoredFetchRun> {
        expect_read("fetch_runs", self.try_fetch_runs())
    }

    fn statement_lines_for_reconciliation_run(&self, run_id: &str) -> Vec<StoredStatementLine> {
        expect_read(
            "statement_lines_for_reconciliation_run",
            self.try_statement_lines_for_reconciliation_run(run_id),
        )
    }

    fn reconciliation_run_count(&self) -> usize {
        expect_read(
            "reconciliation_run_count",
            self.try_reconciliation_run_count(),
        )
    }

    fn reconciliation_run(&self, run_id: &str) -> Option<StoredReconciliationRun> {
        expect_read("reconciliation_run", self.try_reconciliation_run(run_id))
    }

    fn reconciliation_runs(&self) -> Vec<StoredReconciliationRun> {
        expect_read("reconciliation_runs", self.try_reconciliation_runs())
    }

    fn month_close_count(&self) -> usize {
        expect_read("month_close_count", self.try_month_close_count())
    }

    fn month_close(&self, close_id: &str) -> Option<StoredMonthClose> {
        expect_read("month_close", self.try_month_close(close_id))
    }

    fn month_close_for_scope(
        &self,
        month_key: &str,
        checking_account: &str,
    ) -> Option<StoredMonthClose> {
        expect_read(
            "month_close_for_scope",
            self.try_month_close_for_scope(month_key, checking_account),
        )
    }

    fn month_closes(&self) -> Vec<StoredMonthClose> {
        expect_read("month_closes", self.try_month_closes())
    }

    fn transactions_as_of_us(
        &self,
        valid_time_us: i64,
        tx_time_us: i64,
    ) -> Result<Vec<StoredTransaction>, StoreError> {
        let mut connection = self.connection.borrow_mut();
        let superseded_ids: HashSet<String> = corrections::table
            .filter(corrections::recorded_at_us.le(tx_time_us))
            .select(corrections::supersedes_txn_id)
            .load::<String>(&mut *connection)
            .map_err(|err| {
                load_failure(format!("loading superseded transaction ids failed: {err}"))
            })?
            .into_iter()
            .collect();

        let rows = transactions::table
            .filter(transactions::effective_at_us.le(valid_time_us))
            .filter(transactions::recorded_at_us.le(tx_time_us))
            .order(transactions::id.asc())
            .select(TransactionRow::as_select())
            .load::<TransactionRow>(&mut *connection)
            .map_err(|err| load_failure(format!("loading as-of transactions failed: {err}")))?;
        let visible_rows = rows
            .into_iter()
            .filter(|row| !superseded_ids.contains(&row.id))
            .collect();
        Self::hydrate_transactions(&mut connection, visible_rows)
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
        let recorded_at_us = now_timestamp_us()?;
        let effective_at_us = valid_from.unwrap_or(recorded_at_us);
        let mut connection = self.connection.borrow_mut();
        let transaction_id = Self::next_transaction_id(&mut connection)?;
        let transaction_row = NewTransactionRow {
            id: transaction_id.as_str(),
            description: transaction.description(),
            effective_at_us,
            recorded_at_us,
            source_kind: None,
            external_ref: None,
        };
        let posting_rows: Vec<NewPostingRow<'_>> = transaction
            .postings()
            .iter()
            .enumerate()
            .map(|(ordinal, posting): (usize, &Posting)| {
                let ordinal = i32::try_from(ordinal).map_err(|_| {
                    persist_failure(format!(
                        "transaction '{}' has too many postings to persist",
                        transaction_id.as_str()
                    ))
                })?;
                Ok(NewPostingRow {
                    transaction_id: transaction_id.as_str(),
                    ordinal,
                    account: posting.account().as_str(),
                    amount_cents: posting.amount(),
                })
            })
            .collect::<Result<_, StoreError>>()?;

        connection
            .transaction::<(), diesel::result::Error, _>(|conn| {
                diesel::insert_into(transactions::table)
                    .values(&transaction_row)
                    .execute(conn)?;
                diesel::insert_into(postings::table)
                    .values(&posting_rows)
                    .execute(conn)?;
                Ok(())
            })
            .map_err(|err| {
                persist_failure(format!(
                    "persisting transaction '{}' failed: {err}",
                    transaction_id.as_str()
                ))
            })?;

        Ok(transaction_id)
    }

    fn write_correction(&mut self, correction: Correction) -> Result<(), StoreError> {
        if !self.try_has_transaction(correction.supersedes_id())? {
            return Err(StoreError::UnknownTransaction {
                transaction_id: correction.supersedes_id().clone(),
            });
        }

        let correction_row = NewCorrectionRow {
            supersedes_txn_id: correction.supersedes_id().as_str(),
            reason: correction.reason(),
            recorded_at_us: now_timestamp_us()?,
        };
        let mut connection = self.connection.borrow_mut();
        diesel::insert_into(corrections::table)
            .values(&correction_row)
            .execute(&mut *connection)
            .map_err(|err| {
                persist_failure(format!(
                    "persisting correction for '{}' failed: {err}",
                    correction.supersedes_id().as_str()
                ))
            })?;
        Ok(())
    }

    fn write_budget_target(
        &mut self,
        month_key: &str,
        expense_account_prefix: &str,
        budget_cents: i64,
    ) -> Result<(), StoreError> {
        let row = NewBudgetTargetRow {
            month_key,
            expense_account_prefix,
            budget_cents,
        };
        let mut connection = self.connection.borrow_mut();
        diesel::insert_into(budget_targets::table)
            .values(&row)
            .on_conflict((
                budget_targets::month_key,
                budget_targets::expense_account_prefix,
            ))
            .do_update()
            .set(budget_targets::budget_cents.eq(budget_cents))
            .execute(&mut *connection)
            .map_err(|err| persist_failure(format!("persisting budget target failed: {err}")))?;
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
        if let Some(artifact_id) = supersedes_artifact_id {
            if self.try_analytics_artifact(artifact_id)?.is_none() {
                return Err(StoreError::UnknownArtifact {
                    artifact_id: artifact_id.to_owned(),
                });
            }
        }

        let created_at_us = now_timestamp_us()?;
        let mut connection = self.connection.borrow_mut();
        let artifact_id = Self::next_artifact_id(&mut connection)?;
        let snapshot_key = format!("valid:{snapshot_valid_at_us}|tx:{snapshot_tx_at_us}");
        let row = NewAnalyticsArtifactRow {
            artifact_id: &artifact_id,
            artifact_kind,
            artifact_uri,
            content_hash,
            schema_version,
            row_count,
            snapshot_valid_at_us,
            snapshot_tx_at_us,
            created_at_us,
            supersedes_artifact_id,
            snapshot_key: &snapshot_key,
        };
        diesel::insert_into(analytics_artifact_manifests::table)
            .values(&row)
            .execute(&mut *connection)
            .map_err(|err| {
                persist_failure(format!("persisting analytics artifact failed: {err}"))
            })?;
        Ok(StoredAnalyticsArtifactManifest::new(
            &artifact_id,
            artifact_kind,
            artifact_uri,
            content_hash,
            schema_version,
            row_count,
            snapshot_valid_at_us,
            snapshot_tx_at_us,
            created_at_us,
            supersedes_artifact_id,
            &snapshot_key,
        ))
    }

    #[allow(clippy::too_many_lines)]
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
        self.try_validate_import_records(
            import_kind,
            source_uri,
            batch_key,
            duplicate_count,
            records,
        )?;

        let imported_txn_ids: Vec<TransactionId> = records
            .iter()
            .filter_map(|record| record.imported_txn_id().cloned())
            .collect();
        let imported_at_us = now_timestamp_us()?;
        let record_count = i64::try_from(records.len()).unwrap_or(i64::MAX);
        let mut connection = self.connection.borrow_mut();
        Self::ensure_transactions_exist(&mut connection, &imported_txn_ids)?;

        let batch_id = Self::next_import_batch_id(&mut connection)?;
        let batch_row = NewImportBatchRow {
            batch_id: &batch_id,
            import_kind,
            source_uri,
            batch_key,
            record_count,
            duplicate_count,
            dry_run,
            ocr_enabled,
            imported_at_us,
        };
        let import_record_rows: Vec<NewImportRecordRow<'_>> = records
            .iter()
            .map(|record| NewImportRecordRow {
                content_hash_key: record.content_hash_key(),
                batch_id: &batch_id,
                imported_txn_id: record.imported_txn_id().map(TransactionId::as_str),
                imported_at_us,
            })
            .collect();

        // ⚡ Bolt: Pre-allocate statement line payloads to match the upper bound of the import
        // records to avoid dynamic vector reallocations during persistence.
        let mut statement_line_payloads = Vec::with_capacity(records.len());
        for record in records {
            if let Some(line) = record.statement_line() {
                let line_id = Self::next_statement_line_id(&mut connection)?;
                statement_line_payloads.push((
                    line_id,
                    line.source_uri().to_owned(),
                    line.statement_timestamp().to_owned(),
                    line.memo().to_owned(),
                    line.amount_cents(),
                    record.imported_txn_id().cloned(),
                ));
            }
        }
        let statement_line_rows: Vec<NewStatementLineRow<'_>> = statement_line_payloads
            .iter()
            .map(
                |(
                    line_id,
                    source_uri,
                    statement_timestamp,
                    memo,
                    amount_cents,
                    imported_txn_id,
                )| {
                    Self::map_statement_line(
                        line_id.as_str(),
                        source_uri.as_str(),
                        statement_timestamp.as_str(),
                        memo.as_str(),
                        *amount_cents,
                        imported_txn_id.as_ref(),
                        &batch_id,
                        imported_at_us,
                    )
                },
            )
            .collect();

        connection
            .transaction::<(), diesel::result::Error, _>(|conn| {
                diesel::insert_into(import_batches::table)
                    .values(&batch_row)
                    .execute(conn)?;
                if !import_record_rows.is_empty() {
                    diesel::insert_into(import_records::table)
                        .values(&import_record_rows)
                        .execute(conn)?;
                }
                if !statement_line_rows.is_empty() {
                    diesel::insert_into(statement_lines::table)
                        .values(&statement_line_rows)
                        .execute(conn)?;
                }
                Ok(())
            })
            .map_err(|err| persist_failure(format!("persisting import batch failed: {err}")))?;

        Ok(StoredImportBatch::new(
            &batch_id,
            import_kind,
            source_uri,
            batch_key,
            record_count,
            duplicate_count,
            dry_run,
            ocr_enabled,
            imported_at_us,
        ))
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
            return Err(persist_failure("source_id must not be empty".to_owned()));
        }
        if institution_id.is_empty() {
            return Err(persist_failure(
                "institution_id must not be empty".to_owned(),
            ));
        }
        if ledger_account.is_empty() {
            return Err(persist_failure(
                "ledger_account must not be empty".to_owned(),
            ));
        }
        if month_key.is_empty() {
            return Err(persist_failure("month_key must not be empty".to_owned()));
        }
        if matches!(
            status,
            StoredFetchRunStatus::Downloaded | StoredFetchRunStatus::Imported
        ) && (artifact_path.is_none()
            || output_format.is_none()
            || opening_balance_cents.is_none()
            || closing_balance_cents.is_none())
        {
            return Err(persist_failure(format!(
                "status '{}' requires artifact path, format, and balance metadata",
                status.as_str()
            )));
        }

        let created_at_us = now_timestamp_us()?;
        let normalized_error_summary = error_summary.filter(|value| !value.is_empty());
        let mut connection = self.connection.borrow_mut();
        let run_id = Self::next_fetch_run_id(&mut connection)?;
        let row = NewFetchRunRow {
            run_id: &run_id,
            source_id,
            institution_id,
            ledger_account,
            month_key,
            status: status.as_str(),
            artifact_path,
            output_format: output_format.map(StoredFetchArtifactFormat::as_str),
            opening_balance_cents,
            closing_balance_cents,
            error_summary: normalized_error_summary,
            created_at_us,
        };
        diesel::insert_into(fetch_runs::table)
            .values(&row)
            .execute(&mut *connection)
            .map_err(|err| persist_failure(format!("persisting fetch run failed: {err}")))?;
        Ok(StoredFetchRun::new(
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
            created_at_us,
        ))
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
        Self::try_validate_reconciliation_run_params(
            month_key,
            checking_account,
            matched_postings,
            inflow_cents,
            outflow_cents,
        )?;

        let created_at_us = now_timestamp_us()?;
        let matched_transaction_count = i64::try_from(reconciled_txn_ids.len()).unwrap_or(i64::MAX);
        let mut connection = self.connection.borrow_mut();
        Self::ensure_transactions_exist(&mut connection, reconciled_txn_ids)?;
        let statement_line_ids =
            Self::statement_line_ids_for_transaction_ids(&mut connection, reconciled_txn_ids)?;
        let run_id = Self::next_reconciliation_run_id(&mut connection)?;
        let run_row = NewReconciliationRunRow {
            run_id: &run_id,
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
            created_at_us,
        };
        let transaction_rows: Vec<NewReconciliationRunTransactionRow<'_>> = reconciled_txn_ids
            .iter()
            .map(|txn_id| NewReconciliationRunTransactionRow {
                run_id: &run_id,
                transaction_id: txn_id.as_str(),
            })
            .collect();
        let statement_line_rows: Vec<NewReconciliationRunStatementLineRow<'_>> = statement_line_ids
            .iter()
            .map(|line_id| NewReconciliationRunStatementLineRow {
                run_id: &run_id,
                statement_line_id: line_id.as_str(),
            })
            .collect();

        connection
            .transaction::<(), diesel::result::Error, _>(|conn| {
                diesel::insert_into(reconciliation_runs::table)
                    .values(&run_row)
                    .execute(conn)?;
                if !transaction_rows.is_empty() {
                    diesel::insert_into(reconciliation_run_transactions::table)
                        .values(&transaction_rows)
                        .execute(conn)?;
                }
                if !statement_line_rows.is_empty() {
                    diesel::insert_into(reconciliation_run_statement_lines::table)
                        .values(&statement_line_rows)
                        .execute(conn)?;
                }
                Ok(())
            })
            .map_err(|err| {
                persist_failure(format!("persisting reconciliation run failed: {err}"))
            })?;

        Ok(StoredReconciliationRun::new(
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
            created_at_us,
        ))
    }

    #[allow(clippy::too_many_lines)]
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
        Self::try_validate_reconciliation_run_params(
            month_key,
            checking_account,
            matched_postings,
            inflow_cents,
            outflow_cents,
        )?;
        if let Some(artifact_id) = analytics_artifact_id {
            if self.try_analytics_artifact(artifact_id)?.is_none() {
                return Err(StoreError::UnknownArtifact {
                    artifact_id: artifact_id.to_owned(),
                });
            }
        }
        if let Some(existing_close) = self.try_month_close_for_scope(month_key, checking_account)? {
            return Err(persist_failure(format!(
                "month '{month_key}' for account '{checking_account}' is already closed by '{}'",
                existing_close.close_id()
            )));
        }

        let created_at_us = now_timestamp_us()?;
        let closed_at_us = now_timestamp_us()?;
        let matched_transaction_count = i64::try_from(reconciled_txn_ids.len()).unwrap_or(i64::MAX);
        let mut connection = self.connection.borrow_mut();
        Self::ensure_transactions_exist(&mut connection, reconciled_txn_ids)?;
        let statement_line_ids =
            Self::statement_line_ids_for_transaction_ids(&mut connection, reconciled_txn_ids)?;
        let run_id = Self::next_reconciliation_run_id(&mut connection)?;
        let close_id = Self::next_month_close_id(&mut connection)?;

        let run_row = NewReconciliationRunRow {
            run_id: &run_id,
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
            created_at_us,
        };
        let close_row = NewMonthCloseRow {
            close_id: &close_id,
            month_key,
            checking_account,
            reconciliation_run_id: &run_id,
            analytics_artifact_id,
            closed_at_us,
        };
        let transaction_rows: Vec<NewReconciliationRunTransactionRow<'_>> = reconciled_txn_ids
            .iter()
            .map(|txn_id| NewReconciliationRunTransactionRow {
                run_id: &run_id,
                transaction_id: txn_id.as_str(),
            })
            .collect();
        let statement_line_rows: Vec<NewReconciliationRunStatementLineRow<'_>> = statement_line_ids
            .iter()
            .map(|line_id| NewReconciliationRunStatementLineRow {
                run_id: &run_id,
                statement_line_id: line_id.as_str(),
            })
            .collect();

        connection
            .transaction::<(), diesel::result::Error, _>(|conn| {
                diesel::insert_into(reconciliation_runs::table)
                    .values(&run_row)
                    .execute(conn)?;
                if !transaction_rows.is_empty() {
                    diesel::insert_into(reconciliation_run_transactions::table)
                        .values(&transaction_rows)
                        .execute(conn)?;
                }
                if !statement_line_rows.is_empty() {
                    diesel::insert_into(reconciliation_run_statement_lines::table)
                        .values(&statement_line_rows)
                        .execute(conn)?;
                }
                diesel::insert_into(month_closes::table)
                    .values(&close_row)
                    .execute(conn)?;
                Ok(())
            })
            .map_err(|err| {
                persist_failure(format!(
                    "persisting reconciliation run and month close failed: {err}"
                ))
            })?;

        Ok(Self::map_stored_reconciliation_and_close(
            &run_id,
            &close_id,
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
            created_at_us,
            closed_at_us,
            analytics_artifact_id,
        ))
    }

    fn write_month_close(
        &mut self,
        month_key: &str,
        checking_account: &str,
        reconciliation_run_id: &str,
        analytics_artifact_id: Option<&str>,
    ) -> Result<StoredMonthClose, StoreError> {
        if month_key.is_empty() {
            return Err(persist_failure("month_key must not be empty".to_owned()));
        }
        if checking_account.is_empty() {
            return Err(persist_failure(
                "checking_account must not be empty".to_owned(),
            ));
        }
        if reconciliation_run_id.is_empty() {
            return Err(persist_failure(
                "reconciliation_run_id must not be empty".to_owned(),
            ));
        }
        let run = self
            .try_reconciliation_run(reconciliation_run_id)?
            .ok_or_else(|| {
                persist_failure(format!(
                    "unknown reconciliation run '{reconciliation_run_id}'"
                ))
            })?;
        if run.month_key() != month_key {
            return Err(persist_failure(format!(
                "month close month '{month_key}' does not match reconciliation run month '{}'",
                run.month_key()
            )));
        }
        if run.checking_account() != checking_account {
            return Err(persist_failure(format!(
                "month close checking_account '{checking_account}' does not match reconciliation run account '{}'",
                run.checking_account()
            )));
        }
        if let Some(artifact_id) = analytics_artifact_id {
            if self.try_analytics_artifact(artifact_id)?.is_none() {
                return Err(StoreError::UnknownArtifact {
                    artifact_id: artifact_id.to_owned(),
                });
            }
        }
        if let Some(existing_close) = self.try_month_close_for_scope(month_key, checking_account)? {
            return Err(persist_failure(format!(
                "month '{month_key}' for account '{checking_account}' is already closed by '{}'",
                existing_close.close_id()
            )));
        }

        let closed_at_us = now_timestamp_us()?;
        let mut connection = self.connection.borrow_mut();
        let close_id = Self::next_month_close_id(&mut connection)?;
        let row = NewMonthCloseRow {
            close_id: &close_id,
            month_key,
            checking_account,
            reconciliation_run_id,
            analytics_artifact_id,
            closed_at_us,
        };
        diesel::insert_into(month_closes::table)
            .values(&row)
            .execute(&mut *connection)
            .map_err(|err| persist_failure(format!("persisting month close failed: {err}")))?;
        Ok(StoredMonthClose::new(
            &close_id,
            month_key,
            checking_account,
            reconciliation_run_id,
            analytics_artifact_id,
            closed_at_us,
        ))
    }
}

fn expect_read<T>(method: &'static str, result: Result<T, StoreError>) -> T {
    result.unwrap_or_else(|err| panic!("PostgresStore::{method} failed: {err}"))
}

const fn load_failure(message: String) -> StoreError {
    StoreError::LoadFailed { message }
}

const fn persist_failure(message: String) -> StoreError {
    StoreError::PersistFailed { message }
}

fn now_timestamp_us() -> Result<i64, StoreError> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|err| persist_failure(format!("system clock is before unix epoch: {err}")))?;
    i64::try_from(duration.as_micros())
        .map_err(|_| persist_failure("current time exceeds i64 microseconds range".to_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connect_rejects_empty_database_url() {
        let err = PostgresStore::connect("   ")
            .err()
            .expect("empty DATABASE_URL must fail");
        assert!(matches!(err, PgStoreError::MissingDatabaseUrl));
    }
}
