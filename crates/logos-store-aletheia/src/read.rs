use std::collections::HashSet;

use aletheiadb::{
    AletheiaDB, EdgeId, Error as DbError, Node, NodeId, StorageError, TemporalError, Timestamp,
};
use logos_core::{Correction, TransactionBuilder, TransactionId};

use crate::{
    map_load_error,
    model::{
        AsOf, StoredAnalyticsArtifactManifest, StoredBudgetTarget, StoredCorrection,
        StoredImportBatch, StoredImportRecord, StoredMonthClose, StoredReconciliationRun,
        StoredStatementLine, StoredTransaction, EDGE_HAS_POSTING, EDGE_SUPERSEDES,
        LABEL_LEDGER_CORRECTION, PROP_ACCOUNT, PROP_AMOUNT_CENTS, PROP_DESCRIPTION,
        PROP_EFFECTIVE_AT_US, PROP_ORDINAL, PROP_SUPERSEDES_TXN_ID, PROP_TXN_ID,
    },
    parse_posting, required_edge_i64_property, required_node_i64_property,
    required_node_string_property, AletheiaStore, StoreError,
};

impl AletheiaStore {
    #[must_use]
    pub fn transaction_count(&self) -> usize {
        self.transactions.len()
    }

    #[must_use]
    pub fn correction_count(&self) -> usize {
        self.corrections.len()
    }

    #[must_use]
    pub fn has_transaction(&self, id: &TransactionId) -> bool {
        self.transactions.contains_key(id)
    }

    #[must_use]
    pub fn latest_correction(&self) -> Option<&Correction> {
        self.corrections.last().map(StoredCorrection::correction)
    }

    pub fn transactions(&self) -> impl Iterator<Item = &StoredTransaction> + '_ {
        self.transactions.values()
    }

    #[must_use]
    pub fn budget_target(
        &self,
        month_key: &str,
        expense_account_prefix: &str,
    ) -> Option<&StoredBudgetTarget> {
        self.budget_targets
            .get(&(month_key.to_owned(), expense_account_prefix.to_owned()))
    }

    pub fn budget_targets(&self) -> impl Iterator<Item = &StoredBudgetTarget> + '_ {
        self.budget_targets.values()
    }

    #[must_use]
    pub fn analytics_artifact(
        &self,
        artifact_id: &str,
    ) -> Option<&StoredAnalyticsArtifactManifest> {
        self.analytics_artifacts.get(artifact_id)
    }

    pub fn analytics_artifacts(
        &self,
    ) -> impl Iterator<Item = &StoredAnalyticsArtifactManifest> + '_ {
        self.analytics_artifacts.values()
    }

    #[must_use]
    pub fn import_record_count(&self) -> usize {
        self.import_records.len()
    }

    #[must_use]
    pub fn has_import_record_content_hash(&self, content_hash_key: &str) -> bool {
        self.import_records.contains_key(content_hash_key)
    }

    pub fn import_records(&self) -> impl Iterator<Item = &StoredImportRecord> + '_ {
        self.import_records.values()
    }

    pub fn import_batches(&self) -> impl Iterator<Item = &StoredImportBatch> + '_ {
        self.import_batches.values()
    }

    #[must_use]
    pub fn statement_line_count(&self) -> usize {
        self.statement_lines.len()
    }

    pub fn statement_lines(&self) -> impl Iterator<Item = &StoredStatementLine> + '_ {
        self.statement_lines.values()
    }

    #[must_use]
    pub fn statement_lines_for_reconciliation_run(
        &self,
        run_id: &str,
    ) -> Vec<&StoredStatementLine> {
        let mut lines = self
            .reconciliation_statement_line_ids
            .get(run_id)
            .into_iter()
            .flatten()
            .filter_map(|line_id| self.statement_lines.get(line_id))
            .collect::<Vec<_>>();
        lines.sort_by(|left, right| left.line_id().cmp(right.line_id()));
        lines
    }

    #[must_use]
    pub fn reconciliation_run_count(&self) -> usize {
        self.reconciliation_runs.len()
    }

    #[must_use]
    pub fn reconciliation_run(&self, run_id: &str) -> Option<&StoredReconciliationRun> {
        self.reconciliation_runs.get(run_id)
    }

    pub fn reconciliation_runs(&self) -> impl Iterator<Item = &StoredReconciliationRun> + '_ {
        self.reconciliation_runs.values()
    }

    #[must_use]
    pub fn month_close_count(&self) -> usize {
        self.month_closes.len()
    }

    #[must_use]
    pub fn month_close(&self, close_id: &str) -> Option<&StoredMonthClose> {
        self.month_closes.get(close_id)
    }

    #[must_use]
    pub fn month_close_for_scope(
        &self,
        month_key: &str,
        checking_account: &str,
    ) -> Option<&StoredMonthClose> {
        let close_id = self
            .month_close_by_scope
            .get(&(month_key.to_owned(), checking_account.to_owned()))?;
        self.month_closes.get(close_id)
    }

    pub fn month_closes(&self) -> impl Iterator<Item = &StoredMonthClose> + '_ {
        self.month_closes.values()
    }

    /// Reconstructs transactions visible at a bi-temporal point in time.
    ///
    /// # Errors
    ///
    /// Returns an error when historical reconstruction fails due to graph corruption
    /// or storage query errors.
    pub fn transactions_as_of(
        &self,
        valid_time: Timestamp,
        tx_time: Timestamp,
    ) -> Result<Vec<StoredTransaction>, StoreError> {
        self.transactions_at(AsOf::new(valid_time, tx_time))
    }

    /// Reconstructs transactions visible at explicit microsecond timestamps.
    ///
    /// # Errors
    ///
    /// Returns an error when historical reconstruction fails due to graph corruption
    /// or storage query errors.
    pub fn transactions_as_of_us(
        &self,
        valid_time_us: i64,
        tx_time_us: i64,
    ) -> Result<Vec<StoredTransaction>, StoreError> {
        self.transactions_as_of(valid_time_us.into(), tx_time_us.into())
    }

    /// Reconstructs transactions visible at a bi-temporal point in time.
    ///
    /// # Errors
    ///
    /// Returns an error when historical reconstruction fails due to graph corruption
    /// or storage query errors.
    pub fn transactions_at(&self, as_of: AsOf) -> Result<Vec<StoredTransaction>, StoreError> {
        let Some(embedded) = self.embedded.as_ref() else {
            return Ok(self.current_projection_without_superseded());
        };

        let superseded_ids = load_superseded_ids_at(&embedded.db, as_of)?;
        // Pre-allocate transactions vector based on exact node count to prevent multiple heap reallocations.
        let mut transactions = Vec::with_capacity(embedded.transaction_nodes.len());

        for (txn_id, txn_node_id) in &embedded.transaction_nodes {
            if superseded_ids.contains(txn_id) {
                continue;
            }

            let Some(txn_node) = get_node_at_as_of(&embedded.db, *txn_node_id, as_of)? else {
                continue;
            };

            let transaction = reconstruct_transaction_at_as_of(
                &embedded.db,
                *txn_node_id,
                txn_id,
                &txn_node,
                as_of,
            )?;
            let effective_at = optional_node_timestamp_property(&txn_node, PROP_EFFECTIVE_AT_US)
                .unwrap_or_else(|| as_of.valid_time());
            transactions.push(StoredTransaction::with_effective_at(
                txn_id.clone(),
                transaction,
                effective_at,
            ));
        }

        transactions.sort_by(|left, right| left.id().as_str().cmp(right.id().as_str()));
        Ok(transactions)
    }

    /// Projects the current state of transactions, excluding those that have been superseded.
    fn current_projection_without_superseded(&self) -> Vec<StoredTransaction> {
        let superseded_ids: HashSet<_> = self
            .corrections
            .iter()
            .map(StoredCorrection::correction)
            .map(logos_core::Correction::supersedes_id)
            .collect();

        let mut transactions: Vec<_> = self
            .transactions
            .values()
            .filter(|stored| !superseded_ids.contains(&stored.id()))
            .cloned()
            .collect();
        transactions.sort_by(|left, right| left.id().as_str().cmp(right.id().as_str()));
        transactions
    }
}

fn load_superseded_ids_at(
    db: &AletheiaDB,
    as_of: AsOf,
) -> Result<HashSet<TransactionId>, StoreError> {
    let correction_node_ids = db.scan_nodes_by_label(LABEL_LEDGER_CORRECTION);
    // Pre-allocate hash set based on known correction node count to eliminate runtime hashing reallocations.
    let mut superseded_ids = HashSet::new();
    for correction_node_id in correction_node_ids {
        let Some(correction_node) = get_node_at_as_of(db, correction_node_id, as_of)? else {
            continue;
        };

        if !has_visible_supersedes_edge(db, correction_node_id, as_of)? {
            continue;
        }

        let supersedes_txn_id =
            required_node_string_property(&correction_node, PROP_SUPERSEDES_TXN_ID)?;
        superseded_ids.insert(TransactionId::new(&supersedes_txn_id));
    }

    Ok(superseded_ids)
}

fn has_visible_supersedes_edge(
    db: &AletheiaDB,
    correction_node_id: NodeId,
    as_of: AsOf,
) -> Result<bool, StoreError> {
    for edge_id in
        db.get_outgoing_edges_at_time(correction_node_id, as_of.valid_time(), as_of.tx_time())
    {
        let Some(edge) = get_edge_at_as_of(db, edge_id, as_of)? else {
            continue;
        };

        if edge.has_label_str(EDGE_SUPERSEDES) {
            return Ok(true);
        }
    }

    Ok(false)
}

fn reconstruct_transaction_at_as_of(
    db: &AletheiaDB,
    transaction_node_id: NodeId,
    expected_id: &TransactionId,
    transaction_node: &Node,
    as_of: AsOf,
) -> Result<logos_core::Transaction, StoreError> {
    let expected_txn_id = expected_id.as_str();
    let description = required_node_string_property(transaction_node, PROP_DESCRIPTION)?;

    let mut postings = Vec::new();
    for edge_id in
        db.get_outgoing_edges_at_time(transaction_node_id, as_of.valid_time(), as_of.tx_time())
    {
        let Some(edge) = get_edge_at_as_of(db, edge_id, as_of)? else {
            continue;
        };
        if !edge.has_label_str(EDGE_HAS_POSTING) {
            continue;
        }

        let ordinal = required_edge_i64_property(&edge, PROP_ORDINAL)?;
        let Some(posting_node) = get_node_at_as_of(db, edge.target, as_of)? else {
            continue;
        };

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

    postings.sort_by_key(|(ordinal, edge_id, _)| (*ordinal, *edge_id));

    let mut builder = TransactionBuilder::new(&description);
    for (_, _, posting) in postings {
        builder = builder.posting(posting);
    }

    builder.build().map_err(StoreError::Domain)
}

fn get_node_at_as_of(
    db: &AletheiaDB,
    node_id: NodeId,
    as_of: AsOf,
) -> Result<Option<Node>, StoreError> {
    match db.get_node_at_time(node_id, as_of.valid_time(), as_of.tx_time()) {
        Ok(node) => Ok(Some(node)),
        Err(error) if is_node_not_visible(&error) => Ok(None),
        Err(error) => Err(map_load_error("unable to read node at as-of time", error)),
    }
}

fn get_edge_at_as_of(
    db: &AletheiaDB,
    edge_id: EdgeId,
    as_of: AsOf,
) -> Result<Option<aletheiadb::Edge>, StoreError> {
    match db.get_edge_at_time(edge_id, as_of.valid_time(), as_of.tx_time()) {
        Ok(edge) => Ok(Some(edge)),
        Err(error) if is_edge_not_visible(&error) => Ok(None),
        Err(error) => Err(map_load_error("unable to read edge at as-of time", error)),
    }
}

const fn is_node_not_visible(error: &DbError) -> bool {
    matches!(
        error,
        DbError::Storage(StorageError::NodeNotFound(_))
            | DbError::Temporal(TemporalError::NodeNotFoundAtTime { .. })
    )
}

const fn is_edge_not_visible(error: &DbError) -> bool {
    matches!(error, DbError::Storage(StorageError::EdgeNotFound(_)))
}

fn optional_node_timestamp_property(node: &Node, key: &str) -> Option<Timestamp> {
    node.get_property(key)
        .and_then(aletheiadb::PropertyValue::as_int)
        .map(Into::into)
}
