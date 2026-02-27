use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::path::Path;

use aletheiadb::{
    AletheiaDB, AletheiaDBConfig, DurabilityMode, Edge, Node, NodeId, PropertyMapBuilder,
    Timestamp, WalConfigBuilder, WriteOps,
};
use logos_core::{Correction, DomainError, Posting, TransactionBuilder, TransactionId};

use crate::model::{
    EDGE_DERIVED_FROM, EDGE_HAS_IMPORT_RECORD, EDGE_HAS_POSTING, EDGE_RECONCILES_TXN,
    EDGE_SUPERSEDES, LABEL_ANALYTICS_ARTIFACT_MANIFEST, LABEL_LEDGER_BUDGET_TARGET,
    LABEL_LEDGER_CORRECTION, LABEL_LEDGER_IMPORT_BATCH, LABEL_LEDGER_IMPORT_RECORD,
    LABEL_LEDGER_POSTING, LABEL_LEDGER_RECONCILIATION_RUN, LABEL_LEDGER_TRANSACTION,
    NewImportRecord, PROP_ACCOUNT, PROP_AMOUNT_CENTS, PROP_ARTIFACT_ID, PROP_ARTIFACT_KIND,
    PROP_ARTIFACT_URI, PROP_BUDGET_CENTS, PROP_CONTENT_HASH, PROP_CREATED_AT_US, PROP_DESCRIPTION,
    PROP_EFFECTIVE_AT_US, PROP_EXPENSE_ACCOUNT_PREFIX, PROP_IMPORT_BATCH_ID, PROP_IMPORT_BATCH_KEY,
    PROP_IMPORT_CONTENT_HASH_KEY, PROP_IMPORT_DRY_RUN, PROP_IMPORT_DUPLICATE_COUNT,
    PROP_IMPORT_IMPORTED_AT_US, PROP_IMPORT_IMPORTED_TXN_ID, PROP_IMPORT_KIND,
    PROP_IMPORT_OCR_ENABLED, PROP_IMPORT_RECORD_COUNT, PROP_IMPORT_SOURCE_URI, PROP_MONTH_KEY,
    PROP_ORDINAL, PROP_REASON, PROP_RECONCILIATION_CHECKING_ACCOUNT,
    PROP_RECONCILIATION_CREATED_AT_US, PROP_RECONCILIATION_EXPECTED_CLOSING_BALANCE_CENTS,
    PROP_RECONCILIATION_INFLOW_CENTS, PROP_RECONCILIATION_LEDGER_DELTA_CENTS,
    PROP_RECONCILIATION_MATCHED_POSTINGS, PROP_RECONCILIATION_MATCHED_TRANSACTION_COUNT,
    PROP_RECONCILIATION_OPENING_BALANCE_CENTS, PROP_RECONCILIATION_OUTFLOW_CENTS,
    PROP_RECONCILIATION_RECONCILED, PROP_RECONCILIATION_RUN_ID,
    PROP_RECONCILIATION_STATEMENT_CLOSING_BALANCE_CENTS, PROP_RECONCILIATION_VARIANCE_CENTS,
    PROP_ROW_COUNT, PROP_SCHEMA_VERSION, PROP_SNAPSHOT_KEY, PROP_SNAPSHOT_TX_AT_US,
    PROP_SNAPSHOT_VALID_AT_US, PROP_SUPERSEDES_ARTIFACT_ID, PROP_SUPERSEDES_TXN_ID, PROP_TXN_ID,
    StoredAnalyticsArtifactManifest, StoredBudgetTarget, StoredCorrection, StoredImportBatch,
    StoredImportRecord, StoredReconciliationRun, StoredTransaction,
};

pub mod model;
pub mod read;
pub mod write;

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
    pub(crate) next_reconciliation_run_id: u64,
    pub(crate) transactions: HashMap<TransactionId, StoredTransaction>,
    pub(crate) corrections: Vec<StoredCorrection>,
    pub(crate) budget_targets: HashMap<BudgetTargetKey, StoredBudgetTarget>,
    pub(crate) analytics_artifacts: HashMap<String, StoredAnalyticsArtifactManifest>,
    pub(crate) import_batches: HashMap<String, StoredImportBatch>,
    pub(crate) import_records: HashMap<String, StoredImportRecord>,
    pub(crate) reconciliation_runs: HashMap<String, StoredReconciliationRun>,
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
    reconciliation_runs: HashMap<String, StoredReconciliationRun>,
    reconciliation_run_nodes: HashMap<String, NodeId>,
}

type BudgetTargetKey = (String, String);

pub(crate) struct EmbeddedStore {
    pub(crate) db: AletheiaDB,
    pub(crate) transaction_nodes: HashMap<TransactionId, NodeId>,
    pub(crate) analytics_artifact_nodes: HashMap<String, NodeId>,
    pub(crate) import_batch_nodes: HashMap<String, NodeId>,
    pub(crate) reconciliation_run_nodes: HashMap<String, NodeId>,
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
            .field(
                "reconciliation_run_nodes",
                &self.reconciliation_run_nodes.len(),
            )
            .finish()
    }
}

impl fmt::Debug for AletheiaStore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AletheiaStore")
            .field("next_id", &self.next_id)
            .field("next_artifact_id", &self.next_artifact_id)
            .field("next_import_batch_id", &self.next_import_batch_id)
            .field(
                "next_reconciliation_run_id",
                &self.next_reconciliation_run_id,
            )
            .field("transactions", &self.transactions.len())
            .field("corrections", &self.corrections.len())
            .field("budget_targets", &self.budget_targets.len())
            .field("analytics_artifacts", &self.analytics_artifacts.len())
            .field("import_batches", &self.import_batches.len())
            .field("import_records", &self.import_records.len())
            .field("reconciliation_runs", &self.reconciliation_runs.len())
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

        let db = open_embedded_db(&root_path)?;
        let loaded = load_projection(&db)?;
        let next_id = infer_next_id(loaded.transactions.keys());
        let next_artifact_id = infer_next_artifact_id(loaded.analytics_artifacts.keys());
        let next_import_batch_id = infer_next_import_batch_id(loaded.import_batches.keys());
        let next_reconciliation_run_id =
            infer_next_reconciliation_run_id(loaded.reconciliation_runs.keys());

        Ok(Self {
            next_id,
            next_artifact_id,
            next_import_batch_id,
            next_reconciliation_run_id,
            transactions: loaded.transactions,
            corrections: loaded.corrections,
            budget_targets: loaded.budget_targets,
            analytics_artifacts: loaded.analytics_artifacts,
            import_batches: loaded.import_batches,
            import_records: loaded.import_records,
            reconciliation_runs: loaded.reconciliation_runs,
            embedded: Some(EmbeddedStore {
                db,
                transaction_nodes: loaded.transaction_nodes,
                analytics_artifact_nodes: loaded.analytics_artifact_nodes,
                import_batch_nodes: loaded.import_batch_nodes,
                reconciliation_run_nodes: loaded.reconciliation_run_nodes,
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

    pub(crate) fn next_reconciliation_run_id(&mut self) -> String {
        self.next_reconciliation_run_id = self.next_reconciliation_run_id.saturating_add(1);
        format!("recon-{}", self.next_reconciliation_run_id)
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

    pub(crate) fn persist_reconciliation_run(&mut self, run: StoredReconciliationRun) {
        self.reconciliation_runs
            .insert(run.run_id().to_owned(), run);
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

    pub(crate) fn persist_import_batch_graph(
        &mut self,
        batch: &StoredImportBatch,
        records: &[NewImportRecord],
    ) -> Result<(), StoreError> {
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

        for record in records {
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
        }

        tx.commit().map_err(|err| {
            map_persist_error("unable to commit embedded import batch write", err)
        })?;

        embedded
            .import_batch_nodes
            .insert(batch.batch_id().to_owned(), batch_node);
        Ok(())
    }

    pub(crate) fn persist_reconciliation_run_graph(
        &mut self,
        run: &StoredReconciliationRun,
        reconciled_txn_ids: &[TransactionId],
    ) -> Result<(), StoreError> {
        let Some(embedded) = self.embedded.as_mut() else {
            return Ok(());
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

        tx.commit().map_err(|err| {
            map_persist_error("unable to commit embedded reconciliation write", err)
        })?;

        embedded
            .reconciliation_run_nodes
            .insert(run.run_id().to_owned(), run_node);
        Ok(())
    }
}

fn open_embedded_db(root_path: &Path) -> Result<AletheiaDB, StoreError> {
    fs::create_dir_all(root_path).map_err(|err| StoreError::LoadFailed {
        message: format!(
            "unable to create store directory '{}': {err}",
            root_path.display()
        ),
    })?;

    let wal_config = WalConfigBuilder::new()
        .wal_dir(root_path.join("wal"))
        .durability_mode(DurabilityMode::Synchronous)
        .build();
    let mut config = AletheiaDBConfig::builder().wal(wal_config).build();
    config.persistence.data_dir = root_path.join("index-data");

    AletheiaDB::with_unified_config(config).map_err(|err| StoreError::LoadFailed {
        message: format!(
            "unable to initialize embedded AletheiaDB at '{}': {err}",
            root_path.display()
        ),
    })
}

fn load_projection(db: &AletheiaDB) -> Result<LoadedProjection, StoreError> {
    let (transactions, transaction_nodes) = load_transactions(db)?;
    let corrections = load_corrections(db, &transaction_nodes)?;
    let budget_targets = load_budget_targets(db)?;
    let (analytics_artifacts, analytics_artifact_nodes) = load_analytics_artifacts(db)?;
    let (import_batches, import_records, import_batch_nodes) = load_import_batches_and_records(db)?;
    let (reconciliation_runs, reconciliation_run_nodes) =
        load_reconciliation_runs(db, &transaction_nodes)?;
    Ok(LoadedProjection {
        transactions,
        transaction_nodes,
        corrections,
        budget_targets,
        analytics_artifacts,
        analytics_artifact_nodes,
        import_batches,
        import_records,
        import_batch_nodes,
        reconciliation_runs,
        reconciliation_run_nodes,
    })
}

type TransactionLoad = (
    HashMap<TransactionId, StoredTransaction>,
    HashMap<TransactionId, NodeId>,
);

fn load_transactions(db: &AletheiaDB) -> Result<TransactionLoad, StoreError> {
    let mut transactions = HashMap::new();
    let mut transaction_nodes = HashMap::new();

    for txn_node_id in db.scan_nodes_by_label(LABEL_LEDGER_TRANSACTION) {
        let txn_node = db
            .get_node(txn_node_id)
            .map_err(|err| map_load_error("unable to read LedgerTransaction node", err))?;

        let txn_id_value = required_node_string_property(&txn_node, PROP_TXN_ID)?;
        let description = required_node_string_property(&txn_node, PROP_DESCRIPTION)?;
        let txn_id = TransactionId::new(&txn_id_value);
        let effective_at = optional_node_i64_property(&txn_node, PROP_EFFECTIVE_AT_US)
            .map_or_else(aletheiadb::time::now, Into::into);

        if transaction_nodes
            .insert(txn_id.clone(), txn_node_id)
            .is_some()
        {
            return Err(StoreError::LoadFailed {
                message: format!("duplicate transaction id '{txn_id_value}' in graph projection"),
            });
        }

        let mut postings = load_transaction_postings(db, txn_node_id, &txn_id_value)?;
        postings.sort_by_key(|(ordinal, edge_id, _)| (*ordinal, *edge_id));

        let mut builder = TransactionBuilder::new(&description);
        for (_, _, posting) in postings {
            builder = builder.posting(posting);
        }

        let transaction = builder.build().map_err(StoreError::Domain)?;
        transactions.insert(
            txn_id.clone(),
            StoredTransaction::with_effective_at(txn_id, transaction, effective_at),
        );
    }

    Ok((transactions, transaction_nodes))
}

fn load_transaction_postings(
    db: &AletheiaDB,
    transaction_node_id: NodeId,
    expected_txn_id: &str,
) -> Result<Vec<(i64, u64, Posting)>, StoreError> {
    let mut postings = Vec::new();
    let edge_ids = db.get_outgoing_edges_with_label(transaction_node_id, EDGE_HAS_POSTING);
    for edge_id in edge_ids {
        let edge = db
            .get_edge(edge_id)
            .map_err(|err| map_load_error("unable to read HAS_POSTING edge", err))?;
        let ordinal = required_edge_i64_property(&edge, PROP_ORDINAL)?;

        let posting_node = db
            .get_node(edge.target)
            .map_err(|err| map_load_error("unable to read LedgerPosting node", err))?;
        let posting_txn_id = required_node_string_property(&posting_node, PROP_TXN_ID)?;
        if posting_txn_id != expected_txn_id {
            return Err(StoreError::LoadFailed {
                message: format!(
                    "posting node {} references txn_id '{}' but parent transaction is '{}'",
                    posting_node.id.as_u64(),
                    posting_txn_id,
                    expected_txn_id
                ),
            });
        }

        let account = required_node_string_property(&posting_node, PROP_ACCOUNT)?;
        let amount_cents = required_node_i64_property(&posting_node, PROP_AMOUNT_CENTS)?;
        let posting = parse_posting(expected_txn_id, &account, amount_cents)?;
        postings.push((ordinal, edge.id.as_u64(), posting));
    }

    Ok(postings)
}

fn load_corrections(
    db: &AletheiaDB,
    transaction_nodes: &HashMap<TransactionId, NodeId>,
) -> Result<Vec<StoredCorrection>, StoreError> {
    let mut corrections = Vec::new();

    for correction_node_id in db.scan_nodes_by_label(LABEL_LEDGER_CORRECTION) {
        let correction_node = db
            .get_node(correction_node_id)
            .map_err(|err| map_load_error("unable to read LedgerCorrection node", err))?;

        let supersedes_txn_id =
            required_node_string_property(&correction_node, PROP_SUPERSEDES_TXN_ID)?;
        let reason = required_node_string_property(&correction_node, PROP_REASON)?;
        let supersedes_id = TransactionId::new(&supersedes_txn_id);
        let expected_target = transaction_nodes
            .get(&supersedes_id)
            .copied()
            .ok_or_else(|| StoreError::LoadFailed {
                message: format!(
                    "correction node {} references unknown transaction '{}'",
                    correction_node.id.as_u64(),
                    supersedes_txn_id
                ),
            })?;

        let supersedes_edges =
            db.get_outgoing_edges_with_label(correction_node_id, EDGE_SUPERSEDES);
        if supersedes_edges.len() != 1 {
            return Err(StoreError::LoadFailed {
                message: format!(
                    "correction node {} has {} SUPERSEDES edges (expected exactly 1)",
                    correction_node.id.as_u64(),
                    supersedes_edges.len()
                ),
            });
        }

        let supersedes_edge = db
            .get_edge(supersedes_edges[0])
            .map_err(|err| map_load_error("unable to read SUPERSEDES edge", err))?;
        if supersedes_edge.target != expected_target {
            return Err(StoreError::LoadFailed {
                message: format!(
                    "correction node {} points to transaction node {}, expected {}",
                    correction_node.id.as_u64(),
                    supersedes_edge.target.as_u64(),
                    expected_target.as_u64()
                ),
            });
        }

        let correction = Correction::new(supersedes_id, &reason).map_err(StoreError::Domain)?;
        corrections.push((
            correction_node.id.as_u64(),
            StoredCorrection::new(correction),
        ));
    }

    corrections.sort_by_key(|(node_id, _)| *node_id);
    Ok(corrections
        .into_iter()
        .map(|(_, correction)| correction)
        .collect())
}

fn load_budget_targets(
    db: &AletheiaDB,
) -> Result<HashMap<BudgetTargetKey, StoredBudgetTarget>, StoreError> {
    let mut latest_by_key: HashMap<BudgetTargetKey, (u64, StoredBudgetTarget)> = HashMap::new();
    for node_id in db.scan_nodes_by_label(LABEL_LEDGER_BUDGET_TARGET) {
        let node = db
            .get_node(node_id)
            .map_err(|err| map_load_error("unable to read LedgerBudgetTarget node", err))?;

        let month_key = required_node_string_property(&node, PROP_MONTH_KEY)?;
        let expense_account_prefix =
            required_node_string_property(&node, PROP_EXPENSE_ACCOUNT_PREFIX)?;
        let budget_cents = required_node_i64_property(&node, PROP_BUDGET_CENTS)?;
        let target = StoredBudgetTarget::new(&month_key, &expense_account_prefix, budget_cents);
        let key = (month_key, expense_account_prefix);

        match latest_by_key.get_mut(&key) {
            Some((latest_node_id, latest_target)) => {
                if node.id.as_u64() > *latest_node_id {
                    *latest_node_id = node.id.as_u64();
                    *latest_target = target;
                }
            }
            None => {
                latest_by_key.insert(key, (node.id.as_u64(), target));
            }
        }
    }

    Ok(latest_by_key
        .into_iter()
        .map(|(key, (_, target))| (key, target))
        .collect())
}

type ArtifactLoad = (
    HashMap<String, StoredAnalyticsArtifactManifest>,
    HashMap<String, NodeId>,
);

fn load_analytics_artifacts(db: &AletheiaDB) -> Result<ArtifactLoad, StoreError> {
    let mut artifacts = HashMap::new();
    let mut artifact_nodes = HashMap::new();

    for node_id in db.scan_nodes_by_label(LABEL_ANALYTICS_ARTIFACT_MANIFEST) {
        let node = db
            .get_node(node_id)
            .map_err(|err| map_load_error("unable to read AnalyticsArtifactManifest node", err))?;

        let artifact_id = required_node_string_property(&node, PROP_ARTIFACT_ID)?;
        let artifact_kind = required_node_string_property(&node, PROP_ARTIFACT_KIND)?;
        let artifact_uri = required_node_string_property(&node, PROP_ARTIFACT_URI)?;
        let content_hash = required_node_string_property(&node, PROP_CONTENT_HASH)?;
        let schema_version = required_node_i64_property(&node, PROP_SCHEMA_VERSION)?;
        let row_count = required_node_i64_property(&node, PROP_ROW_COUNT)?;
        let snapshot_valid_at =
            required_node_i64_property(&node, PROP_SNAPSHOT_VALID_AT_US)?.into();
        let snapshot_tx_at = required_node_i64_property(&node, PROP_SNAPSHOT_TX_AT_US)?.into();
        let created_at = required_node_i64_property(&node, PROP_CREATED_AT_US)?.into();
        let snapshot_key = required_node_string_property(&node, PROP_SNAPSHOT_KEY)?;
        let supersedes_artifact_id =
            optional_node_string_property(&node, PROP_SUPERSEDES_ARTIFACT_ID)
                .filter(|value| !value.is_empty());

        if artifacts.contains_key(&artifact_id) {
            return Err(StoreError::LoadFailed {
                message: format!(
                    "duplicate analytics artifact id '{artifact_id}' in graph projection"
                ),
            });
        }

        let manifest = StoredAnalyticsArtifactManifest::new(
            &artifact_id,
            &artifact_kind,
            &artifact_uri,
            &content_hash,
            schema_version,
            row_count,
            snapshot_valid_at,
            snapshot_tx_at,
            created_at,
            supersedes_artifact_id.as_deref(),
            &snapshot_key,
        );

        artifacts.insert(artifact_id.clone(), manifest);
        artifact_nodes.insert(artifact_id, node.id);
    }

    for (artifact_id, manifest) in &artifacts {
        if let Some(supersedes_id) = manifest.supersedes_artifact_id() {
            let Some(expected_target_node) = artifact_nodes.get(supersedes_id).copied() else {
                return Err(StoreError::LoadFailed {
                    message: format!(
                        "analytics artifact '{artifact_id}' supersedes unknown artifact '{supersedes_id}'"
                    ),
                });
            };
            let source_node =
                artifact_nodes
                    .get(artifact_id)
                    .copied()
                    .ok_or_else(|| StoreError::LoadFailed {
                        message: format!(
                            "analytics artifact '{artifact_id}' is missing internal node mapping"
                        ),
                    })?;

            let has_derived_from_edge = db
                .get_outgoing_edges_with_label(source_node, EDGE_DERIVED_FROM)
                .into_iter()
                .any(|edge_id| {
                    db.get_edge(edge_id)
                        .map(|edge| edge.target == expected_target_node)
                        .unwrap_or(false)
                });
            if !has_derived_from_edge {
                return Err(StoreError::LoadFailed {
                    message: format!(
                        "analytics artifact '{artifact_id}' is missing DERIVED_FROM edge to '{supersedes_id}'"
                    ),
                });
            }
        }
    }

    Ok((artifacts, artifact_nodes))
}

type ImportLoad = (
    HashMap<String, StoredImportBatch>,
    HashMap<String, StoredImportRecord>,
    HashMap<String, NodeId>,
);

fn load_import_batches_and_records(db: &AletheiaDB) -> Result<ImportLoad, StoreError> {
    let (batches, batch_nodes) = load_import_batches(db)?;
    let records = load_import_records(db, &batches, &batch_nodes)?;
    ensure_no_orphan_import_records(db, &records)?;
    Ok((batches, records, batch_nodes))
}

type ImportBatchLoad = (HashMap<String, StoredImportBatch>, HashMap<String, NodeId>);

fn load_import_batches(db: &AletheiaDB) -> Result<ImportBatchLoad, StoreError> {
    let mut batches = HashMap::new();
    let mut batch_nodes = HashMap::new();

    for node_id in db.scan_nodes_by_label(LABEL_LEDGER_IMPORT_BATCH) {
        let node = db
            .get_node(node_id)
            .map_err(|err| map_load_error("unable to read LedgerImportBatch node", err))?;
        let batch_id = required_node_string_property(&node, PROP_IMPORT_BATCH_ID)?;
        let batch = parse_import_batch_node(&node, &batch_id)?;

        if batches.insert(batch_id.clone(), batch).is_some() {
            return Err(StoreError::LoadFailed {
                message: format!("duplicate import batch id '{batch_id}' in graph projection"),
            });
        }
        batch_nodes.insert(batch_id, node.id);
    }

    Ok((batches, batch_nodes))
}

fn parse_import_batch_node(node: &Node, batch_id: &str) -> Result<StoredImportBatch, StoreError> {
    let import_kind = required_node_string_property(node, PROP_IMPORT_KIND)?;
    let source_uri = required_node_string_property(node, PROP_IMPORT_SOURCE_URI)?;
    let batch_key = required_node_string_property(node, PROP_IMPORT_BATCH_KEY)?;
    let record_count = required_node_i64_property(node, PROP_IMPORT_RECORD_COUNT)?;
    let duplicate_count = required_node_i64_property(node, PROP_IMPORT_DUPLICATE_COUNT)?;
    let dry_run = required_node_bool_flag_property(node, PROP_IMPORT_DRY_RUN)?;
    let ocr_enabled = required_node_bool_flag_property(node, PROP_IMPORT_OCR_ENABLED)?;
    let imported_at = required_node_i64_property(node, PROP_IMPORT_IMPORTED_AT_US)?.into();
    if record_count < 0 {
        return Err(StoreError::LoadFailed {
            message: format!("import batch '{batch_id}' has negative record_count {record_count}"),
        });
    }
    if duplicate_count < 0 {
        return Err(StoreError::LoadFailed {
            message: format!(
                "import batch '{batch_id}' has negative duplicate_count {duplicate_count}"
            ),
        });
    }

    Ok(StoredImportBatch::new(
        batch_id,
        &import_kind,
        &source_uri,
        &batch_key,
        record_count,
        duplicate_count,
        dry_run,
        ocr_enabled,
        imported_at,
    ))
}

fn load_import_records(
    db: &AletheiaDB,
    batches: &HashMap<String, StoredImportBatch>,
    batch_nodes: &HashMap<String, NodeId>,
) -> Result<HashMap<String, StoredImportRecord>, StoreError> {
    let mut records = HashMap::new();

    for (batch_id, batch_node_id) in batch_nodes {
        let edge_ids = db.get_outgoing_edges_with_label(*batch_node_id, EDGE_HAS_IMPORT_RECORD);
        validate_import_record_edge_count(batch_id, batches, edge_ids.len())?;
        for edge_id in edge_ids {
            let record = parse_import_record_from_edge(db, edge_id, batch_id)?;
            let key = record.content_hash_key().to_owned();
            if records.insert(key.clone(), record).is_some() {
                return Err(StoreError::LoadFailed {
                    message: format!(
                        "duplicate import record content_hash_key '{key}' in graph projection"
                    ),
                });
            }
        }
    }

    Ok(records)
}

fn validate_import_record_edge_count(
    batch_id: &str,
    batches: &HashMap<String, StoredImportBatch>,
    observed_edge_count: usize,
) -> Result<(), StoreError> {
    let observed_record_count = i64::try_from(observed_edge_count).unwrap_or(i64::MAX);
    let expected_record_count = batches
        .get(batch_id)
        .map_or(0, StoredImportBatch::record_count);
    if observed_record_count != expected_record_count {
        return Err(StoreError::LoadFailed {
            message: format!(
                "import batch '{batch_id}' has {observed_record_count} HAS_IMPORT_RECORD edges but metadata record_count is {expected_record_count}"
            ),
        });
    }
    Ok(())
}

fn parse_import_record_from_edge(
    db: &AletheiaDB,
    edge_id: aletheiadb::EdgeId,
    expected_batch_id: &str,
) -> Result<StoredImportRecord, StoreError> {
    let edge = db
        .get_edge(edge_id)
        .map_err(|err| map_load_error("unable to read HAS_IMPORT_RECORD edge", err))?;
    let node = db
        .get_node(edge.target)
        .map_err(|err| map_load_error("unable to read LedgerImportRecord node", err))?;

    let record_batch_id = required_node_string_property(&node, PROP_IMPORT_BATCH_ID)?;
    if record_batch_id != expected_batch_id {
        return Err(StoreError::LoadFailed {
            message: format!(
                "import record '{}' claims batch '{}' but parent batch is '{}'",
                node.id.as_u64(),
                record_batch_id,
                expected_batch_id
            ),
        });
    }

    let content_hash_key = required_node_string_property(&node, PROP_IMPORT_CONTENT_HASH_KEY)?;
    let imported_txn_id = optional_node_string_property(&node, PROP_IMPORT_IMPORTED_TXN_ID)
        .filter(|value| !value.is_empty())
        .map(|value| TransactionId::new(&value));
    let imported_at = required_node_i64_property(&node, PROP_IMPORT_IMPORTED_AT_US)?.into();

    Ok(StoredImportRecord::new(
        &content_hash_key,
        expected_batch_id,
        imported_txn_id,
        imported_at,
    ))
}

fn ensure_no_orphan_import_records(
    db: &AletheiaDB,
    records: &HashMap<String, StoredImportRecord>,
) -> Result<(), StoreError> {
    for node_id in db.scan_nodes_by_label(LABEL_LEDGER_IMPORT_RECORD) {
        let node = db
            .get_node(node_id)
            .map_err(|err| map_load_error("unable to read LedgerImportRecord node", err))?;
        let content_hash_key = required_node_string_property(&node, PROP_IMPORT_CONTENT_HASH_KEY)?;
        if !records.contains_key(&content_hash_key) {
            return Err(StoreError::LoadFailed {
                message: format!(
                    "import record '{content_hash_key}' is missing HAS_IMPORT_RECORD edge from a batch"
                ),
            });
        }
    }
    Ok(())
}

type ReconciliationLoad = (
    HashMap<String, StoredReconciliationRun>,
    HashMap<String, NodeId>,
);

fn load_reconciliation_runs(
    db: &AletheiaDB,
    transaction_nodes: &HashMap<TransactionId, NodeId>,
) -> Result<ReconciliationLoad, StoreError> {
    let mut runs = HashMap::new();
    let mut run_nodes = HashMap::new();
    let transaction_ids_by_node: HashMap<_, _> = transaction_nodes
        .iter()
        .map(|(txn_id, node_id)| (*node_id, txn_id.clone()))
        .collect();

    for node_id in db.scan_nodes_by_label(LABEL_LEDGER_RECONCILIATION_RUN) {
        let node = db
            .get_node(node_id)
            .map_err(|err| map_load_error("unable to read LedgerReconciliationRun node", err))?;

        let run_id = required_node_string_property(&node, PROP_RECONCILIATION_RUN_ID)?;
        if runs.contains_key(&run_id) {
            return Err(StoreError::LoadFailed {
                message: format!("duplicate reconciliation run id '{run_id}' in graph projection"),
            });
        }

        let month_key = required_node_string_property(&node, PROP_MONTH_KEY)?;
        let checking_account =
            required_node_string_property(&node, PROP_RECONCILIATION_CHECKING_ACCOUNT)?;
        let opening_balance_cents =
            required_node_i64_property(&node, PROP_RECONCILIATION_OPENING_BALANCE_CENTS)?;
        let ledger_delta_cents =
            required_node_i64_property(&node, PROP_RECONCILIATION_LEDGER_DELTA_CENTS)?;
        let expected_closing_balance_cents =
            required_node_i64_property(&node, PROP_RECONCILIATION_EXPECTED_CLOSING_BALANCE_CENTS)?;
        let statement_closing_balance_cents =
            required_node_i64_property(&node, PROP_RECONCILIATION_STATEMENT_CLOSING_BALANCE_CENTS)?;
        let variance_cents = required_node_i64_property(&node, PROP_RECONCILIATION_VARIANCE_CENTS)?;
        let reconciled = required_node_bool_flag_property(&node, PROP_RECONCILIATION_RECONCILED)?;
        let matched_postings =
            required_node_i64_property(&node, PROP_RECONCILIATION_MATCHED_POSTINGS)?;
        let matched_transaction_count =
            required_node_i64_property(&node, PROP_RECONCILIATION_MATCHED_TRANSACTION_COUNT)?;
        let inflow_cents = required_node_i64_property(&node, PROP_RECONCILIATION_INFLOW_CENTS)?;
        let outflow_cents = required_node_i64_property(&node, PROP_RECONCILIATION_OUTFLOW_CENTS)?;
        let created_at =
            required_node_i64_property(&node, PROP_RECONCILIATION_CREATED_AT_US)?.into();
        if matched_postings < 0 {
            return Err(StoreError::LoadFailed {
                message: format!("reconciliation run '{run_id}' has negative matched_postings"),
            });
        }
        if matched_transaction_count < 0 {
            return Err(StoreError::LoadFailed {
                message: format!(
                    "reconciliation run '{run_id}' has negative matched_transaction_count"
                ),
            });
        }

        let edge_ids = db.get_outgoing_edges_with_label(node_id, EDGE_RECONCILES_TXN);
        let observed_txn_count = i64::try_from(edge_ids.len()).unwrap_or(i64::MAX);
        if observed_txn_count != matched_transaction_count {
            return Err(StoreError::LoadFailed {
                message: format!(
                    "reconciliation run '{run_id}' has {observed_txn_count} RECONCILES_TXN edges but metadata matched_transaction_count is {matched_transaction_count}"
                ),
            });
        }

        let mut seen_txn_ids = std::collections::HashSet::new();
        for edge_id in edge_ids {
            let edge = db
                .get_edge(edge_id)
                .map_err(|err| map_load_error("unable to read RECONCILES_TXN edge", err))?;
            let Some(txn_id) = transaction_ids_by_node.get(&edge.target) else {
                return Err(StoreError::LoadFailed {
                    message: format!(
                        "reconciliation run '{run_id}' points to unknown transaction node {}",
                        edge.target.as_u64()
                    ),
                });
            };
            if !seen_txn_ids.insert(txn_id.as_str().to_owned()) {
                return Err(StoreError::LoadFailed {
                    message: format!(
                        "reconciliation run '{run_id}' has duplicate RECONCILES_TXN edge to '{}'",
                        txn_id.as_str()
                    ),
                });
            }
        }

        let run = StoredReconciliationRun::new(
            &run_id,
            &month_key,
            &checking_account,
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
        runs.insert(run_id.clone(), run);
        run_nodes.insert(run_id, node.id);
    }

    Ok((runs, run_nodes))
}

fn required_node_string_property(node: &Node, key: &str) -> Result<String, StoreError> {
    node.get_property(key)
        .and_then(|value| value.as_str())
        .map(str::to_owned)
        .ok_or_else(|| StoreError::LoadFailed {
            message: format!(
                "node {} is missing required string property '{}'",
                node.id.as_u64(),
                key
            ),
        })
}

fn required_node_bool_flag_property(node: &Node, key: &str) -> Result<bool, StoreError> {
    let value = required_node_i64_property(node, key)?;
    match value {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(StoreError::LoadFailed {
            message: format!(
                "node {} has invalid boolean flag '{}' value {}",
                node.id.as_u64(),
                key,
                value
            ),
        }),
    }
}

fn required_node_i64_property(node: &Node, key: &str) -> Result<i64, StoreError> {
    node.get_property(key)
        .and_then(aletheiadb::PropertyValue::as_int)
        .ok_or_else(|| StoreError::LoadFailed {
            message: format!(
                "node {} is missing required integer property '{}'",
                node.id.as_u64(),
                key
            ),
        })
}

fn optional_node_i64_property(node: &Node, key: &str) -> Option<i64> {
    node.get_property(key)
        .and_then(aletheiadb::PropertyValue::as_int)
}

fn optional_node_string_property(node: &Node, key: &str) -> Option<String> {
    node.get_property(key)
        .and_then(|value| value.as_str())
        .map(str::to_owned)
}

fn required_edge_i64_property(edge: &Edge, key: &str) -> Result<i64, StoreError> {
    edge.get_property(key)
        .and_then(aletheiadb::PropertyValue::as_int)
        .ok_or_else(|| StoreError::LoadFailed {
            message: format!(
                "edge {} is missing required integer property '{}'",
                edge.id.as_u64(),
                key
            ),
        })
}

fn parse_posting(txn_id: &str, account: &str, amount_cents: i64) -> Result<Posting, StoreError> {
    if amount_cents >= 0 {
        return Ok(Posting::debit(account, amount_cents));
    }

    let credit_amount = amount_cents
        .checked_abs()
        .ok_or_else(|| StoreError::LoadFailed {
            message: format!(
                "transaction '{txn_id}' contains posting '{account}' with unsupported amount {amount_cents}"
            ),
        })?;
    Ok(Posting::credit(account, credit_amount))
}

fn map_load_error(context: &str, error: impl fmt::Display) -> StoreError {
    StoreError::LoadFailed {
        message: format!("{context}: {error}"),
    }
}

fn map_persist_error(context: &str, error: impl fmt::Display) -> StoreError {
    StoreError::PersistFailed {
        message: format!("{context}: {error}"),
    }
}

fn infer_next_id<'a, I>(ids: I) -> u64
where
    I: Iterator<Item = &'a TransactionId>,
{
    ids.filter_map(|id| {
        id.as_str()
            .strip_prefix("txn-")
            .and_then(|value| value.parse::<u64>().ok())
    })
    .max()
    .unwrap_or(0)
}

fn infer_next_artifact_id<'a, I>(ids: I) -> u64
where
    I: Iterator<Item = &'a String>,
{
    ids.filter_map(|id| {
        id.strip_prefix("artifact-")
            .and_then(|value| value.parse::<u64>().ok())
    })
    .max()
    .unwrap_or(0)
}

fn infer_next_import_batch_id<'a, I>(ids: I) -> u64
where
    I: Iterator<Item = &'a String>,
{
    ids.filter_map(|id| {
        id.strip_prefix("import-batch-")
            .and_then(|value| value.parse::<u64>().ok())
    })
    .max()
    .unwrap_or(0)
}

fn infer_next_reconciliation_run_id<'a, I>(ids: I) -> u64
where
    I: Iterator<Item = &'a String>,
{
    ids.filter_map(|id| {
        id.strip_prefix("recon-")
            .and_then(|value| value.parse::<u64>().ok())
    })
    .max()
    .unwrap_or(0)
}
