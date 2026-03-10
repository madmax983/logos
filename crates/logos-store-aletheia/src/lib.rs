use std::collections::HashMap;
use std::fmt;
use std::path::Path;

use aletheiadb::{
    AletheiaDB, NodeId, PropertyMapBuilder,
    Timestamp, WriteOps,
};
use logos_core::{Correction, DomainError, TransactionBuilder, TransactionId};

use crate::model::{
    EDGE_CLOSES_ANALYTICS_ARTIFACT, EDGE_CLOSES_RECONCILIATION_RUN, EDGE_DERIVED_FROM,
    EDGE_EVIDENCES_TXN, EDGE_HAS_IMPORT_RECORD, EDGE_HAS_POSTING, EDGE_HAS_STATEMENT_LINE,
    EDGE_RECONCILES_STMT_LINE, EDGE_RECONCILES_TXN, EDGE_SUPERSEDES,
    LABEL_ANALYTICS_ARTIFACT_MANIFEST, LABEL_LEDGER_BUDGET_TARGET, LABEL_LEDGER_CORRECTION,
    LABEL_LEDGER_IMPORT_BATCH, LABEL_LEDGER_IMPORT_RECORD, LABEL_LEDGER_MONTH_CLOSE,
    LABEL_LEDGER_POSTING, LABEL_LEDGER_RECONCILIATION_RUN, LABEL_LEDGER_STATEMENT_LINE,
    LABEL_LEDGER_TRANSACTION, NewImportRecord, PROP_ACCOUNT, PROP_AMOUNT_CENTS, PROP_ARTIFACT_ID,
    PROP_ARTIFACT_KIND, PROP_ARTIFACT_URI, PROP_BUDGET_CENTS, PROP_CONTENT_HASH,
    PROP_CREATED_AT_US, PROP_DESCRIPTION, PROP_EFFECTIVE_AT_US, PROP_EXPENSE_ACCOUNT_PREFIX,
    PROP_IMPORT_BATCH_ID, PROP_IMPORT_BATCH_KEY, PROP_IMPORT_CONTENT_HASH_KEY, PROP_IMPORT_DRY_RUN,
    PROP_IMPORT_DUPLICATE_COUNT, PROP_IMPORT_IMPORTED_AT_US, PROP_IMPORT_IMPORTED_TXN_ID,
    PROP_IMPORT_KIND, PROP_IMPORT_OCR_ENABLED, PROP_IMPORT_RECORD_COUNT, PROP_IMPORT_SOURCE_URI,
    PROP_MONTH_CLOSE_ANALYTICS_ARTIFACT_ID, PROP_MONTH_CLOSE_CLOSED_AT_US, PROP_MONTH_CLOSE_ID,
    PROP_MONTH_CLOSE_RECONCILIATION_RUN_ID, PROP_MONTH_KEY, PROP_ORDINAL, PROP_REASON,
    PROP_RECONCILIATION_CHECKING_ACCOUNT, PROP_RECONCILIATION_CREATED_AT_US,
    PROP_RECONCILIATION_EXPECTED_CLOSING_BALANCE_CENTS, PROP_RECONCILIATION_INFLOW_CENTS,
    PROP_RECONCILIATION_LEDGER_DELTA_CENTS, PROP_RECONCILIATION_MATCHED_POSTINGS,
    PROP_RECONCILIATION_MATCHED_TRANSACTION_COUNT, PROP_RECONCILIATION_OPENING_BALANCE_CENTS,
    PROP_RECONCILIATION_OUTFLOW_CENTS, PROP_RECONCILIATION_RECONCILED, PROP_RECONCILIATION_RUN_ID,
    PROP_RECONCILIATION_STATEMENT_CLOSING_BALANCE_CENTS, PROP_RECONCILIATION_VARIANCE_CENTS,
    PROP_ROW_COUNT, PROP_SCHEMA_VERSION, PROP_SNAPSHOT_KEY, PROP_SNAPSHOT_TX_AT_US,
    PROP_SNAPSHOT_VALID_AT_US, PROP_STATEMENT_AMOUNT_CENTS, PROP_STATEMENT_LINE_ID,
    PROP_STATEMENT_MEMO, PROP_STATEMENT_SOURCE_URI, PROP_STATEMENT_TIMESTAMP,
    PROP_SUPERSEDES_ARTIFACT_ID, PROP_SUPERSEDES_TXN_ID, PROP_TXN_ID,
    StoredAnalyticsArtifactManifest, StoredBudgetTarget, StoredCorrection, StoredImportBatch,
    StoredImportRecord, StoredMonthClose, StoredReconciliationRun, StoredStatementLine,
    StoredTransaction,
};

pub mod model;
pub mod read;
pub mod write;
pub(crate) mod init;

pub(crate) use init::{
    collect_statement_line_ids_for_transactions, map_load_error, map_persist_error, parse_posting,
    required_edge_i64_property, required_node_i64_property,
    required_node_string_property,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoreError {
    Domain(DomainError),
    UnknownTransaction { transaction_id: TransactionId },
    UnknownArtifact { artifact_id: String },
    LoadFailed { message: String },
    PersistFailed { message: String },
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Domain(err) => write!(f, "{err}"),
            Self::UnknownTransaction { transaction_id } => {
                write!(
                    f,
                    "cannot apply correction: unknown transaction '{}'",
                    transaction_id.as_str()
                )
            }
            Self::UnknownArtifact { artifact_id } => {
                write!(
                    f,
                    "cannot link analytics artifact: unknown artifact '{artifact_id}'"
                )
            }
            Self::LoadFailed { message } => write!(f, "failed to load store: {message}"),
            Self::PersistFailed { message } => write!(f, "failed to persist store: {message}"),
        }
    }
}

impl std::error::Error for StoreError {}

impl From<DomainError> for StoreError {
    fn from(value: DomainError) -> Self {
        Self::Domain(value)
    }
}

#[derive(Default)]
pub struct AletheiaStore {
    pub(crate) next_id: u64,
    pub(crate) next_artifact_id: u64,
    pub(crate) next_import_batch_id: u64,
    pub(crate) next_statement_line_id: u64,
    pub(crate) next_reconciliation_run_id: u64,
    pub(crate) next_month_close_id: u64,
    pub(crate) transactions: HashMap<TransactionId, StoredTransaction>,
    pub(crate) corrections: Vec<StoredCorrection>,
    pub(crate) budget_targets: HashMap<BudgetTargetKey, StoredBudgetTarget>,
    pub(crate) analytics_artifacts: HashMap<String, StoredAnalyticsArtifactManifest>,
    pub(crate) import_batches: HashMap<String, StoredImportBatch>,
    pub(crate) import_records: HashMap<String, StoredImportRecord>,
    pub(crate) statement_lines: HashMap<String, StoredStatementLine>,
    pub(crate) statement_line_ids_by_txn: HashMap<TransactionId, Vec<String>>,
    pub(crate) reconciliation_runs: HashMap<String, StoredReconciliationRun>,
    pub(crate) reconciliation_statement_line_ids: HashMap<String, Vec<String>>,
    pub(crate) month_closes: HashMap<String, StoredMonthClose>,
    pub(crate) month_close_by_scope: HashMap<BudgetTargetKey, String>,
    pub(crate) embedded: Option<EmbeddedStore>,
}

#[derive(Default)]
struct LoadedProjection {
    transactions: HashMap<TransactionId, StoredTransaction>,
    transaction_nodes: HashMap<TransactionId, NodeId>,
    corrections: Vec<StoredCorrection>,
    budget_targets: HashMap<BudgetTargetKey, StoredBudgetTarget>,
    analytics_artifacts: HashMap<String, StoredAnalyticsArtifactManifest>,
    analytics_artifact_nodes: HashMap<String, NodeId>,
    import_batches: HashMap<String, StoredImportBatch>,
    import_records: HashMap<String, StoredImportRecord>,
    import_batch_nodes: HashMap<String, NodeId>,
    statement_lines: HashMap<String, StoredStatementLine>,
    statement_line_nodes: HashMap<String, NodeId>,
    reconciliation_runs: HashMap<String, StoredReconciliationRun>,
    reconciliation_run_nodes: HashMap<String, NodeId>,
    reconciliation_statement_line_ids: HashMap<String, Vec<String>>,
    month_closes: HashMap<String, StoredMonthClose>,
    month_close_nodes: HashMap<String, NodeId>,
}

type BudgetTargetKey = (String, String);

pub(crate) struct EmbeddedStore {
    pub(crate) db: AletheiaDB,
    pub(crate) transaction_nodes: HashMap<TransactionId, NodeId>,
    pub(crate) analytics_artifact_nodes: HashMap<String, NodeId>,
    pub(crate) import_batch_nodes: HashMap<String, NodeId>,
    pub(crate) statement_line_nodes: HashMap<String, NodeId>,
    pub(crate) reconciliation_run_nodes: HashMap<String, NodeId>,
    pub(crate) month_close_nodes: HashMap<String, NodeId>,
}

impl fmt::Debug for EmbeddedStore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EmbeddedStore")
            .field("db", &"AletheiaDB")
            .field("transaction_nodes", &self.transaction_nodes.len())
            .field(
                "analytics_artifact_nodes",
                &self.analytics_artifact_nodes.len(),
            )
            .field("import_batch_nodes", &self.import_batch_nodes.len())
            .field("statement_line_nodes", &self.statement_line_nodes.len())
            .field(
                "reconciliation_run_nodes",
                &self.reconciliation_run_nodes.len(),
            )
            .field("month_close_nodes", &self.month_close_nodes.len())
            .finish()
    }
}

impl fmt::Debug for AletheiaStore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AletheiaStore")
            .field("next_id", &self.next_id)
            .field("next_artifact_id", &self.next_artifact_id)
            .field("next_import_batch_id", &self.next_import_batch_id)
            .field("next_statement_line_id", &self.next_statement_line_id)
            .field(
                "next_reconciliation_run_id",
                &self.next_reconciliation_run_id,
            )
            .field("next_month_close_id", &self.next_month_close_id)
            .field("transactions", &self.transactions.len())
            .field("corrections", &self.corrections.len())
            .field("budget_targets", &self.budget_targets.len())
            .field("analytics_artifacts", &self.analytics_artifacts.len())
            .field("import_batches", &self.import_batches.len())
            .field("import_records", &self.import_records.len())
            .field("statement_lines", &self.statement_lines.len())
            .field(
                "statement_line_ids_by_txn",
                &self.statement_line_ids_by_txn.len(),
            )
            .field("reconciliation_runs", &self.reconciliation_runs.len())
            .field(
                "reconciliation_statement_line_ids",
                &self.reconciliation_statement_line_ids.len(),
            )
            .field("month_closes", &self.month_closes.len())
            .field("month_close_by_scope", &self.month_close_by_scope.len())
            .field("embedded", &self.embedded.as_ref().map(|_| "enabled"))
            .finish()
    }
}

impl AletheiaStore {
    #[must_use]
    pub fn new() -> Self {
        Self::new_in_memory()
    }

    #[must_use]
    pub fn new_in_memory() -> Self {
        Self::default()
    }

    /// Opens an embedded durable store rooted at `path`.
    ///
    /// # Errors
    ///
    /// Returns an error when opening `AletheiaDB` or rebuilding the projection fails.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, StoreError> {
        let root_path = path.as_ref().to_path_buf();
        if root_path.is_file() {
            return Err(StoreError::LoadFailed {
                message: format!(
                    "store path '{}' points to a file; expected a directory",
                    root_path.display()
                ),
            });
        }

        let db = init::open_embedded_db(&root_path)?;
        let loaded = init::load_projection(&db)?;
        let next_id = init::infer_next_id(loaded.transactions.keys());
        let next_artifact_id = init::infer_next_artifact_id(loaded.analytics_artifacts.keys());
        let next_import_batch_id = init::infer_next_import_batch_id(loaded.import_batches.keys());
        let next_statement_line_id = init::infer_next_statement_line_id(loaded.statement_lines.keys());
        let next_reconciliation_run_id =
            init::infer_next_reconciliation_run_id(loaded.reconciliation_runs.keys());
        let next_month_close_id = init::infer_next_month_close_id(loaded.month_closes.keys());
        let statement_line_ids_by_txn =
            init::index_statement_lines_by_transaction(&loaded.statement_lines);
        let month_close_by_scope = init::index_month_close_by_scope(&loaded.month_closes);

        Ok(Self {
            next_id,
            next_artifact_id,
            next_import_batch_id,
            next_statement_line_id,
            next_reconciliation_run_id,
            next_month_close_id,
            transactions: loaded.transactions,
            corrections: loaded.corrections,
            budget_targets: loaded.budget_targets,
            analytics_artifacts: loaded.analytics_artifacts,
            import_batches: loaded.import_batches,
            import_records: loaded.import_records,
            statement_lines: loaded.statement_lines,
            statement_line_ids_by_txn,
            reconciliation_runs: loaded.reconciliation_runs,
            reconciliation_statement_line_ids: loaded.reconciliation_statement_line_ids,
            month_closes: loaded.month_closes,
            month_close_by_scope,
            embedded: Some(EmbeddedStore {
                db,
                transaction_nodes: loaded.transaction_nodes,
                analytics_artifact_nodes: loaded.analytics_artifact_nodes,
                import_batch_nodes: loaded.import_batch_nodes,
                statement_line_nodes: loaded.statement_line_nodes,
                reconciliation_run_nodes: loaded.reconciliation_run_nodes,
                month_close_nodes: loaded.month_close_nodes,
            }),
        })
    }

    pub(crate) fn next_transaction_id(&mut self) -> TransactionId {
        self.next_id = self.next_id.saturating_add(1);
        TransactionId::new(&format!("txn-{}", self.next_id))
    }

    pub(crate) fn next_artifact_id(&mut self) -> String {
        self.next_artifact_id = self.next_artifact_id.saturating_add(1);
        format!("artifact-{}", self.next_artifact_id)
    }

    pub(crate) fn next_import_batch_id(&mut self) -> String {
        self.next_import_batch_id = self.next_import_batch_id.saturating_add(1);
        format!("import-batch-{}", self.next_import_batch_id)
    }

    pub(crate) fn next_statement_line_id(&mut self) -> String {
        self.next_statement_line_id = self.next_statement_line_id.saturating_add(1);
        format!("stmt-line-{}", self.next_statement_line_id)
    }

    pub(crate) fn next_reconciliation_run_id(&mut self) -> String {
        self.next_reconciliation_run_id = self.next_reconciliation_run_id.saturating_add(1);
        format!("recon-{}", self.next_reconciliation_run_id)
    }

    pub(crate) fn next_month_close_id(&mut self) -> String {
        self.next_month_close_id = self.next_month_close_id.saturating_add(1);
        format!("close-{}", self.next_month_close_id)
    }

    pub(crate) fn push_correction(&mut self, correction: Correction) {
        self.corrections.push(StoredCorrection::new(correction));
    }

    pub(crate) fn persist_transaction(
        &mut self,
        id: TransactionId,
        txn: logos_core::Transaction,
        effective_at: Timestamp,
    ) {
        self.transactions.insert(
            id.clone(),
            StoredTransaction::with_effective_at(id, txn, effective_at),
        );
    }

    pub(crate) fn persist_budget_target(&mut self, target: StoredBudgetTarget) {
        let key = (
            target.month_key().to_owned(),
            target.expense_account_prefix().to_owned(),
        );
        self.budget_targets.insert(key, target);
    }

    pub(crate) fn persist_analytics_artifact(&mut self, manifest: StoredAnalyticsArtifactManifest) {
        self.analytics_artifacts
            .insert(manifest.artifact_id().to_owned(), manifest);
    }

    pub(crate) fn persist_import_batch(&mut self, batch: StoredImportBatch) {
        self.import_batches
            .insert(batch.batch_id().to_owned(), batch);
    }

    pub(crate) fn persist_import_record(&mut self, record: StoredImportRecord) {
        self.import_records
            .insert(record.content_hash_key().to_owned(), record);
    }

    pub(crate) fn persist_statement_line(&mut self, line: StoredStatementLine) {
        let line_id = line.line_id().to_owned();
        if let Some(txn_id) = line.imported_txn_id().cloned() {
            self.statement_line_ids_by_txn
                .entry(txn_id)
                .or_default()
                .push(line_id.clone());
        }
        self.statement_lines.insert(line_id, line);
    }

    pub(crate) fn persist_reconciliation_run(&mut self, run: StoredReconciliationRun) {
        self.reconciliation_runs
            .insert(run.run_id().to_owned(), run);
    }

    pub(crate) fn persist_reconciliation_statement_line_ids(
        &mut self,
        run_id: &str,
        statement_line_ids: Vec<String>,
    ) {
        self.reconciliation_statement_line_ids
            .insert(run_id.to_owned(), statement_line_ids);
    }

    pub(crate) fn persist_month_close(&mut self, close: StoredMonthClose) {
        self.month_close_by_scope.insert(
            (
                close.month_key().to_owned(),
                close.checking_account().to_owned(),
            ),
            close.close_id().to_owned(),
        );
        self.month_closes.insert(close.close_id().to_owned(), close);
    }

    pub(crate) fn build_and_validate(
        builder: TransactionBuilder,
    ) -> Result<logos_core::Transaction, StoreError> {
        Ok(builder.build()?)
    }

    pub(crate) fn persist_transaction_graph(
        &mut self,
        id: &TransactionId,
        transaction: &logos_core::Transaction,
        effective_at: Timestamp,
    ) -> Result<(), StoreError> {
        let Some(embedded) = self.embedded.as_mut() else {
            return Ok(());
        };

        let mut tx = embedded
            .db
            .write_transaction()
            .map_err(|err| map_persist_error("unable to start embedded write transaction", err))?;

        let txn_node = tx
            .create_node_with_valid_time(
                LABEL_LEDGER_TRANSACTION,
                PropertyMapBuilder::new()
                    .insert(PROP_TXN_ID, id.as_str())
                    .insert(PROP_DESCRIPTION, transaction.description())
                    .insert(PROP_EFFECTIVE_AT_US, effective_at.wallclock())
                    .build(),
                Some(effective_at),
            )
            .map_err(|err| map_persist_error("unable to create LedgerTransaction node", err))?;

        for (index, posting) in transaction.postings().iter().enumerate() {
            let ordinal = i64::try_from(index).map_err(|_| StoreError::PersistFailed {
                message: format!("posting index {index} exceeds i64 range"),
            })?;

            let posting_node = tx
                .create_node_with_valid_time(
                    LABEL_LEDGER_POSTING,
                    PropertyMapBuilder::new()
                        .insert(PROP_TXN_ID, id.as_str())
                        .insert(PROP_ACCOUNT, posting.account())
                        .insert(PROP_AMOUNT_CENTS, posting.amount())
                        .build(),
                    Some(effective_at),
                )
                .map_err(|err| map_persist_error("unable to create LedgerPosting node", err))?;

            tx.create_edge_with_valid_time(
                txn_node,
                posting_node,
                EDGE_HAS_POSTING,
                PropertyMapBuilder::new()
                    .insert(PROP_ORDINAL, ordinal)
                    .build(),
                Some(effective_at),
            )
            .map_err(|err| map_persist_error("unable to create HAS_POSTING edge", err))?;
        }

        tx.commit()
            .map_err(|err| map_persist_error("unable to commit embedded transaction write", err))?;

        embedded.transaction_nodes.insert(id.clone(), txn_node);
        Ok(())
    }

    pub(crate) fn persist_correction_graph(
        &mut self,
        correction: &Correction,
    ) -> Result<(), StoreError> {
        let Some(embedded) = self.embedded.as_mut() else {
            return Ok(());
        };

        let superseded_node = embedded
            .transaction_nodes
            .get(correction.supersedes_id())
            .copied()
            .ok_or_else(|| StoreError::UnknownTransaction {
                transaction_id: correction.supersedes_id().clone(),
            })?;

        let mut tx = embedded.db.write_transaction().map_err(|err| {
            map_persist_error("unable to start correction write transaction", err)
        })?;

        let correction_node = tx
            .create_node(
                LABEL_LEDGER_CORRECTION,
                PropertyMapBuilder::new()
                    .insert(PROP_SUPERSEDES_TXN_ID, correction.supersedes_id().as_str())
                    .insert(PROP_REASON, correction.reason())
                    .build(),
            )
            .map_err(|err| map_persist_error("unable to create LedgerCorrection node", err))?;

        tx.create_edge(
            correction_node,
            superseded_node,
            EDGE_SUPERSEDES,
            PropertyMapBuilder::new().build(),
        )
        .map_err(|err| map_persist_error("unable to create SUPERSEDES edge", err))?;

        tx.commit()
            .map_err(|err| map_persist_error("unable to commit embedded correction write", err))?;
        Ok(())
    }

    pub(crate) fn persist_budget_target_graph(
        &mut self,
        target: &StoredBudgetTarget,
    ) -> Result<(), StoreError> {
        let Some(embedded) = self.embedded.as_mut() else {
            return Ok(());
        };

        let mut tx = embedded.db.write_transaction().map_err(|err| {
            map_persist_error("unable to start budget target write transaction", err)
        })?;

        tx.create_node(
            LABEL_LEDGER_BUDGET_TARGET,
            PropertyMapBuilder::new()
                .insert(PROP_MONTH_KEY, target.month_key())
                .insert(PROP_EXPENSE_ACCOUNT_PREFIX, target.expense_account_prefix())
                .insert(PROP_BUDGET_CENTS, target.budget_cents())
                .build(),
        )
        .map_err(|err| map_persist_error("unable to create LedgerBudgetTarget node", err))?;

        tx.commit()
            .map_err(|err| map_persist_error("unable to commit embedded budget target write", err))
    }

    pub(crate) fn persist_analytics_artifact_graph(
        &mut self,
        manifest: &StoredAnalyticsArtifactManifest,
    ) -> Result<(), StoreError> {
        let Some(embedded) = self.embedded.as_mut() else {
            return Ok(());
        };

        let mut tx = embedded.db.write_transaction().map_err(|err| {
            map_persist_error("unable to start analytics artifact write transaction", err)
        })?;

        let manifest_node = tx
            .create_node(
                LABEL_ANALYTICS_ARTIFACT_MANIFEST,
                PropertyMapBuilder::new()
                    .insert(PROP_ARTIFACT_ID, manifest.artifact_id())
                    .insert(PROP_ARTIFACT_KIND, manifest.artifact_kind())
                    .insert(PROP_ARTIFACT_URI, manifest.artifact_uri())
                    .insert(PROP_CONTENT_HASH, manifest.content_hash())
                    .insert(PROP_SCHEMA_VERSION, manifest.schema_version())
                    .insert(PROP_ROW_COUNT, manifest.row_count())
                    .insert(
                        PROP_SNAPSHOT_VALID_AT_US,
                        manifest.snapshot_valid_at().wallclock(),
                    )
                    .insert(
                        PROP_SNAPSHOT_TX_AT_US,
                        manifest.snapshot_tx_at().wallclock(),
                    )
                    .insert(PROP_CREATED_AT_US, manifest.created_at().wallclock())
                    .insert(PROP_SNAPSHOT_KEY, manifest.snapshot_key())
                    .insert(
                        PROP_SUPERSEDES_ARTIFACT_ID,
                        manifest.supersedes_artifact_id().unwrap_or(""),
                    )
                    .build(),
            )
            .map_err(|err| {
                map_persist_error("unable to create AnalyticsArtifactManifest node", err)
            })?;

        if let Some(supersedes_artifact_id) = manifest.supersedes_artifact_id() {
            let superseded_node = embedded
                .analytics_artifact_nodes
                .get(supersedes_artifact_id)
                .copied()
                .ok_or_else(|| StoreError::UnknownArtifact {
                    artifact_id: supersedes_artifact_id.to_owned(),
                })?;

            tx.create_edge(
                manifest_node,
                superseded_node,
                EDGE_DERIVED_FROM,
                PropertyMapBuilder::new().build(),
            )
            .map_err(|err| map_persist_error("unable to create DERIVED_FROM edge", err))?;
        }

        tx.commit().map_err(|err| {
            map_persist_error("unable to commit embedded analytics artifact write", err)
        })?;

        embedded
            .analytics_artifact_nodes
            .insert(manifest.artifact_id().to_owned(), manifest_node);
        Ok(())
    }

    #[allow(clippy::too_many_lines)]
    pub(crate) fn persist_import_batch_graph(
        &mut self,
        batch: &StoredImportBatch,
        records: &[NewImportRecord],
        statement_lines: &[Option<StoredStatementLine>],
    ) -> Result<(), StoreError> {
        if records.len() != statement_lines.len() {
            return Err(StoreError::PersistFailed {
                message: format!(
                    "import batch '{}' received {} records but {} statement lines",
                    batch.batch_id(),
                    records.len(),
                    statement_lines.len()
                ),
            });
        }

        let Some(embedded) = self.embedded.as_mut() else {
            return Ok(());
        };

        let mut tx = embedded.db.write_transaction().map_err(|err| {
            map_persist_error("unable to start import batch write transaction", err)
        })?;

        let batch_node = tx
            .create_node(
                LABEL_LEDGER_IMPORT_BATCH,
                PropertyMapBuilder::new()
                    .insert(PROP_IMPORT_BATCH_ID, batch.batch_id())
                    .insert(PROP_IMPORT_KIND, batch.import_kind())
                    .insert(PROP_IMPORT_SOURCE_URI, batch.source_uri())
                    .insert(PROP_IMPORT_BATCH_KEY, batch.batch_key())
                    .insert(PROP_IMPORT_RECORD_COUNT, batch.record_count())
                    .insert(PROP_IMPORT_DUPLICATE_COUNT, batch.duplicate_count())
                    .insert(PROP_IMPORT_DRY_RUN, i64::from(batch.dry_run()))
                    .insert(PROP_IMPORT_OCR_ENABLED, i64::from(batch.ocr_enabled()))
                    .insert(PROP_IMPORT_IMPORTED_AT_US, batch.imported_at().wallclock())
                    .build(),
            )
            .map_err(|err| map_persist_error("unable to create LedgerImportBatch node", err))?;

        // Pre-allocate vector based on known record count to prevent multiple heap reallocations.
        let mut statement_line_nodes = Vec::with_capacity(records.len());
        for (record, statement_line) in records.iter().zip(statement_lines.iter()) {
            let record_node = tx
                .create_node(
                    LABEL_LEDGER_IMPORT_RECORD,
                    PropertyMapBuilder::new()
                        .insert(PROP_IMPORT_BATCH_ID, batch.batch_id())
                        .insert(PROP_IMPORT_CONTENT_HASH_KEY, record.content_hash_key())
                        .insert(
                            PROP_IMPORT_IMPORTED_TXN_ID,
                            record.imported_txn_id().map_or("", TransactionId::as_str),
                        )
                        .insert(PROP_IMPORT_IMPORTED_AT_US, batch.imported_at().wallclock())
                        .build(),
                )
                .map_err(|err| {
                    map_persist_error("unable to create LedgerImportRecord node", err)
                })?;

            tx.create_edge(
                batch_node,
                record_node,
                EDGE_HAS_IMPORT_RECORD,
                PropertyMapBuilder::new().build(),
            )
            .map_err(|err| map_persist_error("unable to create HAS_IMPORT_RECORD edge", err))?;

            if let Some(line) = statement_line {
                let line_node = tx
                    .create_node(
                        LABEL_LEDGER_STATEMENT_LINE,
                        PropertyMapBuilder::new()
                            .insert(PROP_STATEMENT_LINE_ID, line.line_id())
                            .insert(PROP_IMPORT_BATCH_ID, line.batch_id())
                            .insert(PROP_STATEMENT_SOURCE_URI, line.source_uri())
                            .insert(PROP_STATEMENT_TIMESTAMP, line.statement_timestamp())
                            .insert(PROP_STATEMENT_MEMO, line.memo())
                            .insert(PROP_STATEMENT_AMOUNT_CENTS, line.amount_cents())
                            .insert(
                                PROP_IMPORT_IMPORTED_TXN_ID,
                                line.imported_txn_id().map_or("", TransactionId::as_str),
                            )
                            .insert(PROP_IMPORT_IMPORTED_AT_US, line.imported_at().wallclock())
                            .build(),
                    )
                    .map_err(|err| {
                        map_persist_error("unable to create LedgerStatementLine node", err)
                    })?;

                tx.create_edge(
                    record_node,
                    line_node,
                    EDGE_HAS_STATEMENT_LINE,
                    PropertyMapBuilder::new().build(),
                )
                .map_err(|err| {
                    map_persist_error("unable to create HAS_STATEMENT_LINE edge", err)
                })?;

                if let Some(txn_id) = line.imported_txn_id() {
                    let txn_node =
                        embedded
                            .transaction_nodes
                            .get(txn_id)
                            .copied()
                            .ok_or_else(|| StoreError::UnknownTransaction {
                                transaction_id: txn_id.clone(),
                            })?;
                    tx.create_edge(
                        line_node,
                        txn_node,
                        EDGE_EVIDENCES_TXN,
                        PropertyMapBuilder::new().build(),
                    )
                    .map_err(|err| map_persist_error("unable to create EVIDENCES_TXN edge", err))?;
                }

                statement_line_nodes.push((line.line_id().to_owned(), line_node));
            }
        }

        tx.commit().map_err(|err| {
            map_persist_error("unable to commit embedded import batch write", err)
        })?;

        embedded
            .import_batch_nodes
            .insert(batch.batch_id().to_owned(), batch_node);
        for (line_id, line_node) in statement_line_nodes {
            embedded.statement_line_nodes.insert(line_id, line_node);
        }
        Ok(())
    }

    pub(crate) fn persist_reconciliation_run_graph(
        &mut self,
        run: &StoredReconciliationRun,
        reconciled_txn_ids: &[TransactionId],
    ) -> Result<Vec<String>, StoreError> {
        let statement_line_ids = collect_statement_line_ids_for_transactions(
            &self.statement_line_ids_by_txn,
            reconciled_txn_ids,
        );

        let Some(embedded) = self.embedded.as_mut() else {
            return Ok(statement_line_ids);
        };

        let mut tx = embedded.db.write_transaction().map_err(|err| {
            map_persist_error("unable to start reconciliation write transaction", err)
        })?;

        let run_node = tx
            .create_node(
                LABEL_LEDGER_RECONCILIATION_RUN,
                PropertyMapBuilder::new()
                    .insert(PROP_RECONCILIATION_RUN_ID, run.run_id())
                    .insert(PROP_MONTH_KEY, run.month_key())
                    .insert(PROP_RECONCILIATION_CHECKING_ACCOUNT, run.checking_account())
                    .insert(
                        PROP_RECONCILIATION_OPENING_BALANCE_CENTS,
                        run.opening_balance_cents(),
                    )
                    .insert(
                        PROP_RECONCILIATION_LEDGER_DELTA_CENTS,
                        run.ledger_delta_cents(),
                    )
                    .insert(
                        PROP_RECONCILIATION_EXPECTED_CLOSING_BALANCE_CENTS,
                        run.expected_closing_balance_cents(),
                    )
                    .insert(
                        PROP_RECONCILIATION_STATEMENT_CLOSING_BALANCE_CENTS,
                        run.statement_closing_balance_cents(),
                    )
                    .insert(PROP_RECONCILIATION_VARIANCE_CENTS, run.variance_cents())
                    .insert(PROP_RECONCILIATION_RECONCILED, i64::from(run.reconciled()))
                    .insert(PROP_RECONCILIATION_MATCHED_POSTINGS, run.matched_postings())
                    .insert(
                        PROP_RECONCILIATION_MATCHED_TRANSACTION_COUNT,
                        run.matched_transaction_count(),
                    )
                    .insert(PROP_RECONCILIATION_INFLOW_CENTS, run.inflow_cents())
                    .insert(PROP_RECONCILIATION_OUTFLOW_CENTS, run.outflow_cents())
                    .insert(
                        PROP_RECONCILIATION_CREATED_AT_US,
                        run.created_at().wallclock(),
                    )
                    .build(),
            )
            .map_err(|err| {
                map_persist_error("unable to create LedgerReconciliationRun node", err)
            })?;

        for txn_id in reconciled_txn_ids {
            let txn_node = embedded
                .transaction_nodes
                .get(txn_id)
                .copied()
                .ok_or_else(|| StoreError::UnknownTransaction {
                    transaction_id: txn_id.clone(),
                })?;
            tx.create_edge(
                run_node,
                txn_node,
                EDGE_RECONCILES_TXN,
                PropertyMapBuilder::new().build(),
            )
            .map_err(|err| map_persist_error("unable to create RECONCILES_TXN edge", err))?;
        }

        for line_id in &statement_line_ids {
            let line_node = embedded
                .statement_line_nodes
                .get(line_id)
                .copied()
                .ok_or_else(|| StoreError::PersistFailed {
                    message: format!(
                        "reconciliation run '{}' references unknown statement line '{}'",
                        run.run_id(),
                        line_id
                    ),
                })?;
            tx.create_edge(
                run_node,
                line_node,
                EDGE_RECONCILES_STMT_LINE,
                PropertyMapBuilder::new().build(),
            )
            .map_err(|err| map_persist_error("unable to create RECONCILES_STMT_LINE edge", err))?;
        }

        tx.commit().map_err(|err| {
            map_persist_error("unable to commit embedded reconciliation write", err)
        })?;

        embedded
            .reconciliation_run_nodes
            .insert(run.run_id().to_owned(), run_node);
        Ok(statement_line_ids)
    }

    #[allow(clippy::too_many_lines)]
    pub(crate) fn persist_reconciliation_run_and_month_close_graph(
        &mut self,
        run: &StoredReconciliationRun,
        close: &StoredMonthClose,
        reconciled_txn_ids: &[TransactionId],
    ) -> Result<Vec<String>, StoreError> {
        let statement_line_ids = collect_statement_line_ids_for_transactions(
            &self.statement_line_ids_by_txn,
            reconciled_txn_ids,
        );

        let Some(embedded) = self.embedded.as_mut() else {
            return Ok(statement_line_ids);
        };

        if close.reconciliation_run_id() != run.run_id() {
            return Err(StoreError::PersistFailed {
                message: format!(
                    "month close '{}' references reconciliation run '{}' but payload run is '{}'",
                    close.close_id(),
                    close.reconciliation_run_id(),
                    run.run_id()
                ),
            });
        }

        let mut tx = embedded.db.write_transaction().map_err(|err| {
            map_persist_error(
                "unable to start reconciliation + month close write transaction",
                err,
            )
        })?;
        let run_node = tx
            .create_node(
                LABEL_LEDGER_RECONCILIATION_RUN,
                PropertyMapBuilder::new()
                    .insert(PROP_RECONCILIATION_RUN_ID, run.run_id())
                    .insert(PROP_MONTH_KEY, run.month_key())
                    .insert(PROP_RECONCILIATION_CHECKING_ACCOUNT, run.checking_account())
                    .insert(
                        PROP_RECONCILIATION_OPENING_BALANCE_CENTS,
                        run.opening_balance_cents(),
                    )
                    .insert(
                        PROP_RECONCILIATION_LEDGER_DELTA_CENTS,
                        run.ledger_delta_cents(),
                    )
                    .insert(
                        PROP_RECONCILIATION_EXPECTED_CLOSING_BALANCE_CENTS,
                        run.expected_closing_balance_cents(),
                    )
                    .insert(
                        PROP_RECONCILIATION_STATEMENT_CLOSING_BALANCE_CENTS,
                        run.statement_closing_balance_cents(),
                    )
                    .insert(PROP_RECONCILIATION_VARIANCE_CENTS, run.variance_cents())
                    .insert(PROP_RECONCILIATION_RECONCILED, i64::from(run.reconciled()))
                    .insert(PROP_RECONCILIATION_MATCHED_POSTINGS, run.matched_postings())
                    .insert(
                        PROP_RECONCILIATION_MATCHED_TRANSACTION_COUNT,
                        run.matched_transaction_count(),
                    )
                    .insert(PROP_RECONCILIATION_INFLOW_CENTS, run.inflow_cents())
                    .insert(PROP_RECONCILIATION_OUTFLOW_CENTS, run.outflow_cents())
                    .insert(
                        PROP_RECONCILIATION_CREATED_AT_US,
                        run.created_at().wallclock(),
                    )
                    .build(),
            )
            .map_err(|err| {
                map_persist_error("unable to create LedgerReconciliationRun node", err)
            })?;

        for txn_id in reconciled_txn_ids {
            let txn_node = embedded
                .transaction_nodes
                .get(txn_id)
                .copied()
                .ok_or_else(|| StoreError::UnknownTransaction {
                    transaction_id: txn_id.clone(),
                })?;
            tx.create_edge(
                run_node,
                txn_node,
                EDGE_RECONCILES_TXN,
                PropertyMapBuilder::new().build(),
            )
            .map_err(|err| map_persist_error("unable to create RECONCILES_TXN edge", err))?;
        }

        for line_id in &statement_line_ids {
            let line_node = embedded
                .statement_line_nodes
                .get(line_id)
                .copied()
                .ok_or_else(|| StoreError::PersistFailed {
                    message: format!(
                        "reconciliation run '{}' references unknown statement line '{}'",
                        run.run_id(),
                        line_id
                    ),
                })?;
            tx.create_edge(
                run_node,
                line_node,
                EDGE_RECONCILES_STMT_LINE,
                PropertyMapBuilder::new().build(),
            )
            .map_err(|err| map_persist_error("unable to create RECONCILES_STMT_LINE edge", err))?;
        }

        let close_node = tx
            .create_node(
                LABEL_LEDGER_MONTH_CLOSE,
                PropertyMapBuilder::new()
                    .insert(PROP_MONTH_CLOSE_ID, close.close_id())
                    .insert(PROP_MONTH_KEY, close.month_key())
                    .insert(
                        PROP_RECONCILIATION_CHECKING_ACCOUNT,
                        close.checking_account(),
                    )
                    .insert(
                        PROP_MONTH_CLOSE_RECONCILIATION_RUN_ID,
                        close.reconciliation_run_id(),
                    )
                    .insert(
                        PROP_MONTH_CLOSE_ANALYTICS_ARTIFACT_ID,
                        close.analytics_artifact_id().unwrap_or(""),
                    )
                    .insert(PROP_MONTH_CLOSE_CLOSED_AT_US, close.closed_at().wallclock())
                    .build(),
            )
            .map_err(|err| map_persist_error("unable to create LedgerMonthClose node", err))?;
        tx.create_edge(
            close_node,
            run_node,
            EDGE_CLOSES_RECONCILIATION_RUN,
            PropertyMapBuilder::new().build(),
        )
        .map_err(|err| map_persist_error("unable to create CLOSES_RECONCILIATION_RUN edge", err))?;

        if let Some(artifact_id) = close.analytics_artifact_id() {
            let artifact_node = embedded
                .analytics_artifact_nodes
                .get(artifact_id)
                .copied()
                .ok_or_else(|| StoreError::UnknownArtifact {
                    artifact_id: artifact_id.to_owned(),
                })?;
            tx.create_edge(
                close_node,
                artifact_node,
                EDGE_CLOSES_ANALYTICS_ARTIFACT,
                PropertyMapBuilder::new().build(),
            )
            .map_err(|err| {
                map_persist_error("unable to create CLOSES_ANALYTICS_ARTIFACT edge", err)
            })?;
        }

        tx.commit().map_err(|err| {
            map_persist_error(
                "unable to commit embedded reconciliation + month close write",
                err,
            )
        })?;
        embedded
            .reconciliation_run_nodes
            .insert(run.run_id().to_owned(), run_node);
        embedded
            .month_close_nodes
            .insert(close.close_id().to_owned(), close_node);
        Ok(statement_line_ids)
    }

    pub(crate) fn persist_month_close_graph(
        &mut self,
        close: &StoredMonthClose,
    ) -> Result<(), StoreError> {
        let Some(embedded) = self.embedded.as_mut() else {
            return Ok(());
        };

        let run_node = embedded
            .reconciliation_run_nodes
            .get(close.reconciliation_run_id())
            .copied()
            .ok_or_else(|| StoreError::PersistFailed {
                message: format!(
                    "month close '{}' references unknown reconciliation run '{}'",
                    close.close_id(),
                    close.reconciliation_run_id()
                ),
            })?;

        let mut tx = embedded.db.write_transaction().map_err(|err| {
            map_persist_error("unable to start month close write transaction", err)
        })?;
        let close_node = tx
            .create_node(
                LABEL_LEDGER_MONTH_CLOSE,
                PropertyMapBuilder::new()
                    .insert(PROP_MONTH_CLOSE_ID, close.close_id())
                    .insert(PROP_MONTH_KEY, close.month_key())
                    .insert(
                        PROP_RECONCILIATION_CHECKING_ACCOUNT,
                        close.checking_account(),
                    )
                    .insert(
                        PROP_MONTH_CLOSE_RECONCILIATION_RUN_ID,
                        close.reconciliation_run_id(),
                    )
                    .insert(
                        PROP_MONTH_CLOSE_ANALYTICS_ARTIFACT_ID,
                        close.analytics_artifact_id().unwrap_or(""),
                    )
                    .insert(PROP_MONTH_CLOSE_CLOSED_AT_US, close.closed_at().wallclock())
                    .build(),
            )
            .map_err(|err| map_persist_error("unable to create LedgerMonthClose node", err))?;

        tx.create_edge(
            close_node,
            run_node,
            EDGE_CLOSES_RECONCILIATION_RUN,
            PropertyMapBuilder::new().build(),
        )
        .map_err(|err| map_persist_error("unable to create CLOSES_RECONCILIATION_RUN edge", err))?;

        if let Some(artifact_id) = close.analytics_artifact_id() {
            let artifact_node = embedded
                .analytics_artifact_nodes
                .get(artifact_id)
                .copied()
                .ok_or_else(|| StoreError::UnknownArtifact {
                    artifact_id: artifact_id.to_owned(),
                })?;
            tx.create_edge(
                close_node,
                artifact_node,
                EDGE_CLOSES_ANALYTICS_ARTIFACT,
                PropertyMapBuilder::new().build(),
            )
            .map_err(|err| {
                map_persist_error("unable to create CLOSES_ANALYTICS_ARTIFACT edge", err)
            })?;
        }

        tx.commit()
            .map_err(|err| map_persist_error("unable to commit embedded month close write", err))?;
        embedded
            .month_close_nodes
            .insert(close.close_id().to_owned(), close_node);
        Ok(())
    }
}

