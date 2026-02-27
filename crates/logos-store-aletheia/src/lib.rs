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
    EDGE_HAS_POSTING, EDGE_SUPERSEDES, LABEL_LEDGER_CORRECTION, LABEL_LEDGER_POSTING,
    LABEL_LEDGER_TRANSACTION, PROP_ACCOUNT, PROP_AMOUNT_CENTS, PROP_DESCRIPTION, PROP_ORDINAL,
    PROP_REASON, PROP_SUPERSEDES_TXN_ID, PROP_TXN_ID, StoredCorrection, StoredTransaction,
};

pub mod model;
pub mod read;
pub mod write;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoreError {
    Domain(DomainError),
    UnknownTransaction { transaction_id: TransactionId },
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
    pub(crate) transactions: HashMap<TransactionId, StoredTransaction>,
    pub(crate) corrections: Vec<StoredCorrection>,
    pub(crate) embedded: Option<EmbeddedStore>,
}

#[derive(Default)]
struct LoadedProjection {
    transactions: HashMap<TransactionId, StoredTransaction>,
    transaction_nodes: HashMap<TransactionId, NodeId>,
    corrections: Vec<StoredCorrection>,
}

pub(crate) struct EmbeddedStore {
    pub(crate) db: AletheiaDB,
    pub(crate) transaction_nodes: HashMap<TransactionId, NodeId>,
}

impl fmt::Debug for EmbeddedStore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EmbeddedStore")
            .field("db", &"AletheiaDB")
            .field("transaction_nodes", &self.transaction_nodes.len())
            .finish()
    }
}

impl fmt::Debug for AletheiaStore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AletheiaStore")
            .field("next_id", &self.next_id)
            .field("transactions", &self.transactions.len())
            .field("corrections", &self.corrections.len())
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

        Ok(Self {
            next_id,
            transactions: loaded.transactions,
            corrections: loaded.corrections,
            embedded: Some(EmbeddedStore {
                db,
                transaction_nodes: loaded.transaction_nodes,
            }),
        })
    }

    pub(crate) fn next_transaction_id(&mut self) -> TransactionId {
        self.next_id = self.next_id.saturating_add(1);
        TransactionId::new(&format!("txn-{}", self.next_id))
    }

    pub(crate) fn push_correction(&mut self, correction: Correction) {
        self.corrections.push(StoredCorrection::new(correction));
    }

    pub(crate) fn persist_transaction(&mut self, id: TransactionId, txn: logos_core::Transaction) {
        self.transactions
            .insert(id.clone(), StoredTransaction::new(id, txn));
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
        valid_from: Option<Timestamp>,
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
                    .build(),
                valid_from,
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
                    valid_from,
                )
                .map_err(|err| map_persist_error("unable to create LedgerPosting node", err))?;

            tx.create_edge_with_valid_time(
                txn_node,
                posting_node,
                EDGE_HAS_POSTING,
                PropertyMapBuilder::new()
                    .insert(PROP_ORDINAL, ordinal)
                    .build(),
                valid_from,
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
    Ok(LoadedProjection {
        transactions,
        transaction_nodes,
        corrections,
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
        transactions.insert(txn_id.clone(), StoredTransaction::new(txn_id, transaction));
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
