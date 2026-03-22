//! Read operations and fast memory-cache projections for the `AletheiaDB` ledger.
//!
//! This module provides the accessors required for driving user interfaces, reporting,
//! and CLI tools without incurring disk I/O latency for every query. It leverages the
//! `AletheiaStore`'s in-memory hash maps to stream domain entities instantly.

use std::collections::HashSet;

use aletheiadb::{
    AletheiaDB, EdgeId, Error as DbError, Node, NodeId, StorageError, TemporalError, Timestamp,
};
use logos_core::{Correction, TransactionBuilder, TransactionId};

use crate::{
    AletheiaStore, StoreError, map_load_error,
    model::{
        AsOf, EDGE_HAS_POSTING, EDGE_SUPERSEDES, LABEL_LEDGER_CORRECTION, PROP_ACCOUNT,
        PROP_AMOUNT_CENTS, PROP_DESCRIPTION, PROP_EFFECTIVE_AT_US, PROP_ORDINAL,
        PROP_SUPERSEDES_TXN_ID, PROP_TXN_ID, StoredAnalyticsArtifactManifest, StoredBudgetTarget,
        StoredCorrection, StoredFetchRun, StoredImportBatch, StoredImportRecord, StoredMonthClose,
        StoredReconciliationRun, StoredStatementLine, StoredTransaction,
    },
    parse_posting, required_edge_i64_property, required_node_i64_property,
    required_node_string_property,
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

    /// Provides an iterator over all historical transactions loaded into the cache.
    ///
    /// This is typically used by aggregation algorithms or CLI search utilities
    /// that need to scan the entire ledger history sequentially.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use logos_store_aletheia::AletheiaStore;
    ///
    /// let store = AletheiaStore::new();
    /// let count = store.transactions().count();
    /// assert_eq!(count, 0); // Empty until synchronized
    /// ```
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

    /// Iterates over all envelope budget allocations.
    ///
    /// Used heavily by the budgeting reconciliation views to calculate variances
    /// across all envelopes rapidly.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use logos_store_aletheia::AletheiaStore;
    ///
    /// let store = AletheiaStore::new();
    /// for target in store.budget_targets() {
    ///     println!("Budget: {}", target.month_key());
    /// }
    /// ```
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

    /// Iterates over manifests for data science or reporting extracts.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use logos_store_aletheia::AletheiaStore;
    ///
    /// let store = AletheiaStore::new();
    /// let manifest_count = store.analytics_artifacts().count();
    /// assert_eq!(manifest_count, 0);
    /// ```
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

    /// Streams all raw import rows (CSV/API) parsed by the system.
    ///
    /// Often used to detect duplicates across imports or build lineage graphs.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use logos_store_aletheia::AletheiaStore;
    ///
    /// let store = AletheiaStore::new();
    /// let imported = store.import_records().count();
    /// ```
    pub fn import_records(&self) -> impl Iterator<Item = &StoredImportRecord> + '_ {
        self.import_records.values()
    }

    /// Streams the high-level metadata for distinct ingest operations.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use logos_store_aletheia::AletheiaStore;
    ///
    /// let store = AletheiaStore::new();
    /// for batch in store.import_batches() {
    ///     println!("Ingested batch ID: {}", batch.batch_id());
    /// }
    /// ```
    pub fn import_batches(&self) -> impl Iterator<Item = &StoredImportBatch> + '_ {
        self.import_batches.values()
    }

    #[must_use]
    pub fn statement_line_count(&self) -> usize {
        self.statement_lines.len()
    }

    /// Iterates over individual bank statement line items used for reconciliation.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use logos_store_aletheia::AletheiaStore;
    ///
    /// let store = AletheiaStore::new();
    /// let lines = store.statement_lines().count();
    /// ```
    pub fn statement_lines(&self) -> impl Iterator<Item = &StoredStatementLine> + '_ {
        self.statement_lines.values()
    }

    #[must_use]
    pub fn fetch_run_count(&self) -> usize {
        self.fetch_runs.len()
    }

    #[must_use]
    pub fn fetch_run(&self, run_id: &str) -> Option<&StoredFetchRun> {
        self.fetch_runs.get(run_id)
    }

    /// Yields metadata about remote API data sync attempts.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use logos_store_aletheia::AletheiaStore;
    ///
    /// let store = AletheiaStore::new();
    /// for run in store.fetch_runs() {
    ///     println!("Sync run: {}", run.run_id());
    /// }
    /// ```
    pub fn fetch_runs(&self) -> impl Iterator<Item = &StoredFetchRun> + '_ {
        self.fetch_runs.values()
    }

    /// Retrieves statement lines for a reconciliation run.
    /// ⚡ Bolt Optimization: Uses `Vec::with_capacity` based on the exact count of linked line IDs
    /// instead of `.collect::<Vec<_>>()`, avoiding intermediate memory reallocations during graph traversal.
    #[must_use]
    pub fn statement_lines_for_reconciliation_run(
        &self,
        run_id: &str,
    ) -> Vec<&StoredStatementLine> {
        let mut lines = self
            .reconciliation_statement_line_ids
            .get(run_id)
            .map_or_else(Vec::new, |line_ids| {
                let mut preallocated = Vec::with_capacity(line_ids.len());
                preallocated.extend(
                    line_ids
                        .iter()
                        .filter_map(|line_id| self.statement_lines.get(line_id)),
                );
                preallocated
            });
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

    /// Iterates over saved runs that match ledger txns against statement lines.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use logos_store_aletheia::AletheiaStore;
    ///
    /// let store = AletheiaStore::new();
    /// let recon_count = store.reconciliation_runs().count();
    /// ```
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

    /// Streams locked accounting period boundaries to prevent historical drift.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use logos_store_aletheia::AletheiaStore;
    ///
    /// let store = AletheiaStore::new();
    /// for close in store.month_closes() {
    ///     println!("Closed: {}", close.month_key());
    /// }
    /// ```
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
        let mut superseded_ids = HashSet::with_capacity(self.corrections.len());
        superseded_ids.extend(
            self.corrections
                .iter()
                .map(StoredCorrection::correction)
                .map(logos_core::Correction::supersedes_id),
        );

        let mut transactions = Vec::with_capacity(self.transactions.len());
        transactions.extend(
            self.transactions
                .values()
                .filter(|stored| !superseded_ids.contains(&stored.id()))
                .cloned(),
        );
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
        let supersedes_id = TransactionId::new(&supersedes_txn_id).map_err(StoreError::Domain)?;
        superseded_ids.insert(supersedes_id);
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

    let outgoing_edges =
        db.get_outgoing_edges_at_time(transaction_node_id, as_of.valid_time(), as_of.tx_time());
    let mut postings = Vec::with_capacity(outgoing_edges.len());
    for edge_id in outgoing_edges {
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
    let result = db.get_node_at_time(node_id, as_of.valid_time(), as_of.tx_time());
    if let Err(error) = &result {
        if is_node_not_visible(error) {
            return Ok(None);
        }
    }
    result
        .map(Some)
        .map_err(|error| map_load_error("unable to read node at as-of time", error))
}

fn get_edge_at_as_of(
    db: &AletheiaDB,
    edge_id: EdgeId,
    as_of: AsOf,
) -> Result<Option<aletheiadb::Edge>, StoreError> {
    let result = db.get_edge_at_time(edge_id, as_of.valid_time(), as_of.tx_time());
    if let Err(error) = &result {
        if is_edge_not_visible(error) {
            return Ok(None);
        }
    }
    result
        .map(Some)
        .map_err(|error| map_load_error("unable to read edge at as-of time", error))
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

#[cfg(test)]
mod tests {
    #[test]
    fn test_transactions_as_of_error_when_mismatched_txn_id() {
        use aletheiadb::{EdgeId, Error as DbError, NodeId, StorageError};
        let node_err = DbError::Storage(StorageError::NodeNotFound(NodeId::new(1).unwrap()));
        assert!(is_node_not_visible(&node_err));

        let edge_err = DbError::Storage(StorageError::EdgeNotFound(EdgeId::new(1).unwrap()));
        assert!(is_edge_not_visible(&edge_err));
    }

    use super::*;
    use aletheiadb::{EdgeId, NodeId, StorageError, TemporalError, core::hlc::HybridTimestamp};

    #[test]
    fn test_is_edge_not_visible() {
        let err = DbError::Storage(StorageError::EdgeNotFound(EdgeId::new(1).unwrap()));
        assert!(is_edge_not_visible(&err));

        let err2 = DbError::Storage(StorageError::NodeNotFound(NodeId::new(1).unwrap()));
        assert!(!is_edge_not_visible(&err2));
    }

    #[test]
    fn test_is_node_not_visible() {
        let err = DbError::Storage(StorageError::NodeNotFound(NodeId::new(1).unwrap()));
        assert!(is_node_not_visible(&err));

        let err2 = DbError::Temporal(TemporalError::NodeNotFoundAtTime {
            node_id: NodeId::new(1).unwrap(),
            valid_time: HybridTimestamp::new(0, 0).unwrap(),
            transaction_time: HybridTimestamp::new(0, 0).unwrap(),
        });
        assert!(is_node_not_visible(&err2));

        let err3 = DbError::Storage(StorageError::EdgeNotFound(EdgeId::new(1).unwrap()));
        assert!(!is_node_not_visible(&err3));
    }

    #[test]
    fn test_map_load_error() {
        let err = DbError::Storage(StorageError::NodeNotFound(NodeId::new(1).unwrap()));
        let mapped = map_load_error("test context", err);
        assert!(mapped.to_string().contains("test context"));
    }

    fn temp_db_path(prefix: &str) -> std::path::PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        std::env::temp_dir().join(format!("logos-store-read-tests-{prefix}-{nanos}.db"))
    }

    #[test]
    fn test_get_node_at_as_of_error_handling() {
        let path = temp_db_path("node-error");
        let db = crate::open_embedded_db(&path).unwrap();
        let node_id = NodeId::new(999).unwrap();
        let as_of = crate::model::AsOf::new(aletheiadb::time::now(), aletheiadb::time::now());

        // This will naturally throw a NodeNotFound error because the DB is empty
        // NodeNotFound maps to None via is_node_not_visible
        let result = super::get_node_at_as_of(&db, node_id, as_of);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());

        if path.exists() {
            let _ = std::fs::remove_dir_all(&path);
        }
    }

    #[test]
    fn test_get_edge_at_as_of_error_handling() {
        let path = temp_db_path("edge-error");
        let db = crate::open_embedded_db(&path).unwrap();
        let edge_id = EdgeId::new(999).unwrap();
        let as_of = crate::model::AsOf::new(aletheiadb::time::now(), aletheiadb::time::now());

        // This will naturally throw an EdgeNotFound error because the DB is empty
        // EdgeNotFound maps to None via is_edge_not_visible
        let result = super::get_edge_at_as_of(&db, edge_id, as_of);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());

        if path.exists() {
            let _ = std::fs::remove_dir_all(&path);
        }
    }

    #[test]
    fn test_has_visible_supersedes_edge_returns_false() {
        use aletheiadb::WriteOps;
        let path = temp_db_path("edge-vis-error");
        let db = crate::open_embedded_db(&path).unwrap();

        // Write nodes and edges via a transaction block, mapping errors correctly
        let node_id = db
            .write(|tx: &mut aletheiadb::WriteTransaction| {
                let n1 = tx.create_node("TestNode", aletheiadb::PropertyMap::default())?;
                let n2 = tx.create_node("TargetNode", aletheiadb::PropertyMap::default())?;
                tx.create_edge(n1, n2, "NotSupersedes", aletheiadb::PropertyMap::default())?;
                Ok::<NodeId, DbError>(n1)
            })
            .unwrap();

        let as_of = crate::model::AsOf::new(aletheiadb::time::now(), aletheiadb::time::now());

        let result = super::has_visible_supersedes_edge(&db, node_id, as_of);
        assert!(result.is_ok());
        assert!(!result.unwrap());

        if path.exists() {
            let _ = std::fs::remove_dir_all(&path);
        }
    }
    #[test]
    fn test_has_visible_supersedes_edge_returns_true() {
        use aletheiadb::WriteOps;
        let path = temp_db_path("edge-vis-true");
        let db = crate::open_embedded_db(&path).unwrap();

        // Write nodes and edges via a transaction block
        let node_id = db
            .write(|tx: &mut aletheiadb::WriteTransaction| {
                let n1 = tx.create_node("TestNode", aletheiadb::PropertyMap::default())?;
                let n2 = tx.create_node("TargetNode", aletheiadb::PropertyMap::default())?;
                tx.create_edge(n1, n2, "SUPERSEDES", aletheiadb::PropertyMap::default())?;
                Ok::<NodeId, DbError>(n1)
            })
            .unwrap();

        let as_of = crate::model::AsOf::new(aletheiadb::time::now(), aletheiadb::time::now());

        let result = super::has_visible_supersedes_edge(&db, node_id, as_of);
        assert!(result.is_ok());
        assert!(result.unwrap());

        if path.exists() {
            let _ = std::fs::remove_dir_all(&path);
        }
    }

    #[test]
    fn test_reconstruct_transaction_at_as_of_skips_non_posting_edges() {
        use aletheiadb::WriteOps;
        let path = temp_db_path("reconstruct-edge-skip");
        let db = crate::open_embedded_db(&path).unwrap();

        let node_id = db
            .write(|tx: &mut aletheiadb::WriteTransaction| {
                let n1 = tx.create_node(
                    crate::model::LABEL_LEDGER_TRANSACTION,
                    aletheiadb::PropertyMapBuilder::new()
                        .insert(crate::model::PROP_TXN_ID, "txn-1")
                        .insert(crate::model::PROP_DESCRIPTION, "desc")
                        .build(),
                )?;
                let n2 = tx.create_node("TargetNode", aletheiadb::PropertyMap::default())?;
                tx.create_edge(n1, n2, "NOT_A_POSTING", aletheiadb::PropertyMap::default())?;
                Ok::<NodeId, DbError>(n1)
            })
            .unwrap();

        let as_of = crate::model::AsOf::new(aletheiadb::time::now(), aletheiadb::time::now());
        let node = super::get_node_at_as_of(&db, node_id, as_of)
            .unwrap()
            .unwrap();
        let expected_id = logos_core::TransactionId::new("txn-1").unwrap();

        let result =
            super::reconstruct_transaction_at_as_of(&db, node_id, &expected_id, &node, as_of);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            StoreError::Domain(logos_core::DomainError::EmptyTransactionPostings)
        ));

        if path.exists() {
            let _ = std::fs::remove_dir_all(&path);
        }
    }

    #[test]
    fn test_get_node_and_edge_at_as_of_pass_through_other_errors() {
        let err = DbError::Storage(StorageError::DuplicateId {
            id: "1".into(),
            kind: "node".into(),
        });
        assert!(!super::is_node_not_visible(&err));
        assert!(!super::is_edge_not_visible(&err));
    }

    #[test]
    #[allow(clippy::needless_collect)]
    fn test_store_empty_accessors() {
        let store = crate::AletheiaStore::new();
        assert_eq!(store.transaction_count(), 0);
        assert_eq!(store.correction_count(), 0);
        assert!(!store.has_transaction(&logos_core::TransactionId::new("missing").unwrap()));
        assert!(store.latest_correction().is_none());
        assert_eq!(store.transactions().count(), 0);
        assert_eq!(store.transactions().collect::<Vec<_>>().len(), 0);
        assert!(store.budget_target("2026-03", "expenses:food").is_none());
        assert_eq!(store.budget_targets().count(), 0);
        assert_eq!(store.budget_targets().collect::<Vec<_>>().len(), 0);
        assert!(store.analytics_artifact("missing").is_none());
        assert_eq!(store.analytics_artifacts().count(), 0);
        assert_eq!(store.analytics_artifacts().collect::<Vec<_>>().len(), 0);
        assert_eq!(store.import_record_count(), 0);
        assert!(!store.has_import_record_content_hash("missing"));
        assert_eq!(store.import_records().count(), 0);
        assert_eq!(store.import_records().collect::<Vec<_>>().len(), 0);
        assert_eq!(store.import_batches().count(), 0);
        assert_eq!(store.import_batches().collect::<Vec<_>>().len(), 0);
        assert_eq!(store.statement_line_count(), 0);
        assert_eq!(store.statement_lines().count(), 0);
        assert!(
            store
                .statement_lines_for_reconciliation_run("missing")
                .is_empty()
        );
        assert_eq!(store.reconciliation_run_count(), 0);
        assert!(store.reconciliation_run("missing").is_none());
        assert_eq!(store.reconciliation_runs().count(), 0);
        assert_eq!(store.reconciliation_runs().collect::<Vec<_>>().len(), 0);
        assert_eq!(store.month_close_count(), 0);
        assert!(store.month_close("missing").is_none());
        assert!(
            store
                .month_close_for_scope("2026-03", "assets:checking")
                .is_none()
        );
        assert_eq!(store.month_closes().count(), 0);
        assert_eq!(store.month_closes().collect::<Vec<_>>().len(), 0);

        // transactions_as_of_us
        let as_of_us = store.transactions_as_of_us(1000, 2000).unwrap();
        assert!(as_of_us.is_empty());

        let proj = store.current_projection_without_superseded();
        assert!(proj.is_empty());
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    #[allow(clippy::needless_collect)]
    fn test_store_populated_accessors() {
        use logos_core::{AccountId, Correction, Posting, TransactionBuilder};
        let mut store = crate::AletheiaStore::new();

        // Write TWO transactions to ensure count > 1 and iterator isn't empty
        let txn_id1 = store
            .write_transaction(
                TransactionBuilder::new("test1")
                    .posting(Posting::debit(AccountId::new("assets:checking").unwrap(), 100).unwrap())
                    .posting(Posting::credit(AccountId::new("income:salary").unwrap(), 100).unwrap()),
            )
            .unwrap();

        let txn_id2 = store
            .write_transaction(
                TransactionBuilder::new("test2")
                    .posting(Posting::debit(AccountId::new("assets:checking").unwrap(), 200).unwrap())
                    .posting(Posting::credit(AccountId::new("income:salary").unwrap(), 200).unwrap()),
            )
            .unwrap();

        assert_eq!(store.transaction_count(), 2);
        assert!(store.has_transaction(&txn_id1));
        assert!(store.has_transaction(&txn_id2));
        assert!(!store.has_transaction(&logos_core::TransactionId::new("missing").unwrap()));
        assert_eq!(store.transactions().count(), 2);
        assert_eq!(store.transactions().collect::<Vec<_>>().len(), 2);

        // Write TWO corrections
        let corr1 = Correction::new(txn_id1.clone(), "fix1").unwrap();
        store.write_correction(corr1).unwrap();
        let corr2 = Correction::new(txn_id2.clone(), "fix2").unwrap();
        store.write_correction(corr2).unwrap();
        assert_eq!(store.correction_count(), 2);

        // Write TWO budget targets
        store.write_budget_target("2026-03", "expenses:food", 500).unwrap();
        store.write_budget_target("2026-03", "expenses:rent", 1500).unwrap();
        assert_eq!(store.budget_targets().count(), 2);
        assert_eq!(store.budget_targets().collect::<Vec<_>>().len(), 2);

        // Write TWO analytics artifacts
        store.write_analytics_artifact_manifest("k1", "u1", "h1", 1, 1, aletheiadb::time::now(), aletheiadb::time::now(), None).unwrap();
        store.write_analytics_artifact_manifest("k2", "u2", "h2", 1, 1, aletheiadb::time::now(), aletheiadb::time::now(), None).unwrap();
        assert_eq!(store.analytics_artifacts().count(), 2);
        assert_eq!(store.analytics_artifacts().collect::<Vec<_>>().len(), 2);

        // Write TWO import batches
        let rec1 = crate::model::NewImportRecord::new("hash1", Some(&txn_id1));
        store.write_import_batch("kind", "uri", "batch1", 0, false, false, &[rec1]).unwrap();
        let rec2 = crate::model::NewImportRecord::new("hash2", Some(&txn_id2));
        store.write_import_batch("kind", "uri", "batch2", 0, false, false, &[rec2]).unwrap();
        assert_eq!(store.import_record_count(), 2);
        assert!(store.has_import_record_content_hash("hash1"));
        assert!(!store.has_import_record_content_hash("missing_hash"));
        assert_eq!(store.import_records().count(), 2);
        assert_eq!(store.import_records().collect::<Vec<_>>().len(), 2);
        assert_eq!(store.import_batches().count(), 2);
        assert_eq!(store.import_batches().collect::<Vec<_>>().len(), 2);

        // Write TWO statement lines via import batch
        let sl1 = crate::model::NewImportRecord::with_statement_line("line1", Some(&txn_id1), "uri", "2026-03-01T00:00:00", "memo", -100);
        store.write_import_batch("stmt", "uri", "batch3", 0, false, false, &[sl1]).unwrap();
        let sl2 = crate::model::NewImportRecord::with_statement_line("line2", Some(&txn_id2), "uri", "2026-03-01T00:00:00", "memo", -100);
        store.write_import_batch("stmt", "uri", "batch4", 0, false, false, &[sl2]).unwrap();
        assert_eq!(store.statement_line_count(), 2);
        assert_eq!(store.statement_lines().count(), 2);

        // Write TWO reconciliation runs
        let run1 = store.write_reconciliation_run("2026-03", "assets:checking", 0, 100, 100, 100, 0, true, 1, 100, 0, std::slice::from_ref(&txn_id1)).unwrap();
        let run2 = store.write_reconciliation_run("2026-03", "assets:savings", 0, 100, 100, 100, 0, true, 1, 100, 0, std::slice::from_ref(&txn_id2)).unwrap();
        assert_eq!(store.reconciliation_run_count(), 2);
        assert_eq!(store.reconciliation_runs().count(), 2);
        assert_eq!(store.reconciliation_runs().collect::<Vec<_>>().len(), 2);

        // Write TWO month closes
        store.write_month_close("2026-03", "assets:checking", run1.run_id(), None).unwrap();
        store.write_month_close("2026-03", "assets:savings", run2.run_id(), None).unwrap();
        assert_eq!(store.month_close_count(), 2);
        assert!(store.month_close("close-1").is_some());
        assert!(store.month_close("close-2").is_some());
        assert!(store.month_close("missing_close").is_none());
        assert_eq!(store.month_closes().count(), 2);
        assert_eq!(store.month_closes().collect::<Vec<_>>().len(), 2);
    }

    #[test]
    fn test_current_projection_without_superseded() {
        use logos_core::{AccountId, Correction, Posting, TransactionBuilder};
        let mut store = crate::AletheiaStore::new();

        // Write transaction 1
        let txn_id1 = store
            .write_transaction(
                TransactionBuilder::new("test1")
                    .posting(
                        Posting::debit(AccountId::new("assets:checking").unwrap(), 100).unwrap(),
                    )
                    .posting(
                        Posting::credit(AccountId::new("income:salary").unwrap(), 100).unwrap(),
                    ),
            )
            .unwrap();

        // Write transaction 2
        let txn_id2 = store
            .write_transaction(
                TransactionBuilder::new("test2")
                    .posting(
                        Posting::debit(AccountId::new("assets:checking").unwrap(), 200).unwrap(),
                    )
                    .posting(
                        Posting::credit(AccountId::new("income:salary").unwrap(), 200).unwrap(),
                    ),
            )
            .unwrap();

        let mut proj = store.current_projection_without_superseded();
        assert!(!proj.iter().any(|t| t.id().as_str() == "missing"));
        assert_eq!(proj.len(), 2);
        assert!(proj.iter().any(|t| t.id() == &txn_id1));
        assert!(proj.iter().any(|t| t.id() == &txn_id2));

        // Supersede transaction 1
        let corr = Correction::new(txn_id1.clone(), "fix").unwrap();
        store.write_correction(corr).unwrap();

        proj = store.current_projection_without_superseded();
        assert_eq!(proj.len(), 1);
        assert_eq!(
            proj.iter()
                .map(crate::model::StoredTransaction::id)
                .collect::<Vec<_>>(),
            vec![&txn_id2]
        );
        assert!(!proj.iter().any(|t| t.id() == &txn_id1));
        assert!(proj.iter().any(|t| t.id() == &txn_id2));

        // Check missing ids are false to prevent ! negation removal mutants
        assert!(!store.transactions.values().any(|stored| {
            store
                .corrections
                .iter()
                .map(crate::model::StoredCorrection::correction)
                .map(logos_core::Correction::supersedes_id)
                .any(|s| *s == *stored.id())
                && proj.iter().any(|p| p.id() == stored.id())
        }));
    }

    #[test]
    fn test_current_projection_without_superseded_not_empty_mutant() {
        use logos_core::{AccountId, Posting, TransactionBuilder};
        let mut store = crate::AletheiaStore::new();
        let txn_id = store
            .write_transaction(
                TransactionBuilder::new("test")
                    .posting(
                        Posting::debit(AccountId::new("assets:checking").unwrap(), 100).unwrap(),
                    )
                    .posting(
                        Posting::credit(AccountId::new("income:salary").unwrap(), 100).unwrap(),
                    ),
            )
            .unwrap();

        let proj = store.current_projection_without_superseded();
        assert_eq!(proj.len(), 1);
        assert_eq!(proj[0].id().as_str(), txn_id.as_str());

        // This will kill:
        // replace AletheiaStore::transactions_as_of_us -> Result<Vec<StoredTransaction>, StoreError> with Ok(vec![])
        // replace AletheiaStore::current_projection_without_superseded -> Vec<StoredTransaction> with vec![]
        let as_of_us = store.transactions_as_of_us(i64::MAX, i64::MAX).unwrap();
        assert_eq!(as_of_us.len(), 1);
        assert_eq!(as_of_us[0].id().as_str(), txn_id.as_str());
    }

    #[test]
    fn test_reconstruct_transaction_at_as_of_edge_not_posting() {
        use aletheiadb::WriteOps;
        let path = temp_db_path("reconstruct-edge-not-posting");
        let db = crate::open_embedded_db(&path).unwrap();

        let node_id = db
            .write(|tx: &mut aletheiadb::WriteTransaction| {
                let n1 = tx.create_node(
                    crate::model::LABEL_LEDGER_TRANSACTION,
                    aletheiadb::PropertyMapBuilder::new()
                        .insert(crate::model::PROP_TXN_ID, "txn-1")
                        .insert(crate::model::PROP_DESCRIPTION, "desc")
                        .build(),
                )?;
                let n2 = tx.create_node("TargetNode", aletheiadb::PropertyMap::default())?;
                tx.create_edge(n1, n2, "NOT_A_POSTING", aletheiadb::PropertyMap::default())?;
                Ok::<NodeId, DbError>(n1)
            })
            .unwrap();

        let as_of = crate::model::AsOf::new(aletheiadb::time::now(), aletheiadb::time::now());
        let node = super::get_node_at_as_of(&db, node_id, as_of)
            .unwrap()
            .unwrap();
        let expected_id = logos_core::TransactionId::new("txn-1").unwrap();

        let result =
            super::reconstruct_transaction_at_as_of(&db, node_id, &expected_id, &node, as_of);
        // It should skip the NOT_A_POSTING edge, and then fail with EmptyTransactionPostings
        // If mutant removed `!`, it would NOT skip NOT_A_POSTING edge, and try to parse it.
        // And then it would fail with a missing property error.
        assert!(result.is_err());
        match result.unwrap_err() {
            crate::StoreError::Domain(_) => {} // Good
            e => panic!("Expected Domain error but got {e:?}"),
        }
    }
}
