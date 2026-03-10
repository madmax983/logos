use crate::model::{
    EDGE_CLOSES_ANALYTICS_ARTIFACT, EDGE_CLOSES_RECONCILIATION_RUN, EDGE_DERIVED_FROM,
    EDGE_EVIDENCES_TXN, EDGE_HAS_IMPORT_RECORD, EDGE_HAS_POSTING, EDGE_HAS_STATEMENT_LINE,
    EDGE_RECONCILES_STMT_LINE, EDGE_RECONCILES_TXN, EDGE_SUPERSEDES,
    LABEL_ANALYTICS_ARTIFACT_MANIFEST, LABEL_LEDGER_BUDGET_TARGET, LABEL_LEDGER_CORRECTION,
    LABEL_LEDGER_IMPORT_BATCH, LABEL_LEDGER_IMPORT_RECORD, LABEL_LEDGER_MONTH_CLOSE, LABEL_LEDGER_RECONCILIATION_RUN, LABEL_LEDGER_STATEMENT_LINE,
    LABEL_LEDGER_TRANSACTION, PROP_ACCOUNT, PROP_AMOUNT_CENTS, PROP_ARTIFACT_ID,
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
use crate::StoreError;
use crate::BudgetTargetKey;
use crate::LoadedProjection;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fs;
use std::path::Path;

use aletheiadb::{
    AletheiaDB, AletheiaDBConfig, DurabilityMode, Edge, Node, NodeId, WalConfigBuilder,
};
use logos_core::{Correction, Posting, TransactionBuilder, TransactionId};

pub fn open_embedded_db(root_path: &Path) -> Result<AletheiaDB, StoreError> {
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

pub fn load_projection(db: &AletheiaDB) -> Result<LoadedProjection, StoreError> {
    let (transactions, transaction_nodes) = load_transactions(db)?;
    let corrections = load_corrections(db, &transaction_nodes)?;
    let budget_targets = load_budget_targets(db)?;
    let (analytics_artifacts, analytics_artifact_nodes) = load_analytics_artifacts(db)?;
    let (import_batches, import_records, import_batch_nodes, statement_lines, statement_line_nodes) =
        load_import_batches_and_records(db, &transaction_nodes)?;
    let (reconciliation_runs, reconciliation_run_nodes, reconciliation_statement_line_ids) =
        load_reconciliation_runs(db, &transaction_nodes, &statement_line_nodes)?;
    let (month_closes, month_close_nodes) =
        load_month_closes(db, &reconciliation_run_nodes, &analytics_artifact_nodes)?;
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
        statement_lines,
        statement_line_nodes,
        reconciliation_runs,
        reconciliation_run_nodes,
        reconciliation_statement_line_ids,
        month_closes,
        month_close_nodes,
    })
}

type TransactionLoad = (
    HashMap<TransactionId, StoredTransaction>,
    HashMap<TransactionId, NodeId>,
);

pub fn load_transactions(db: &AletheiaDB) -> Result<TransactionLoad, StoreError> {
    let txn_node_ids = db.scan_nodes_by_label(LABEL_LEDGER_TRANSACTION);
    // Pre-allocate hash map based on known node count to eliminate runtime hashing reallocations.
    let mut transactions = HashMap::new();
    let mut transaction_nodes = HashMap::new();

    for txn_node_id in txn_node_ids {
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

pub fn load_transaction_postings(
    db: &AletheiaDB,
    transaction_node_id: NodeId,
    expected_txn_id: &str,
) -> Result<Vec<(i64, u64, Posting)>, StoreError> {
    let edge_ids = db.get_outgoing_edges_with_label(transaction_node_id, EDGE_HAS_POSTING);
    // Pre-allocate vector based on known edge count to prevent multiple heap reallocations.
    let mut postings = Vec::with_capacity(edge_ids.len());
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

pub fn load_corrections(
    db: &AletheiaDB,
    transaction_nodes: &HashMap<TransactionId, NodeId>,
) -> Result<Vec<StoredCorrection>, StoreError> {
    let correction_node_ids = db.scan_nodes_by_label(LABEL_LEDGER_CORRECTION);
    // Pre-allocate vector based on known node count to prevent multiple heap reallocations.
    let mut corrections = Vec::new();

    for correction_node_id in correction_node_ids {
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

pub fn load_budget_targets(
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

pub fn load_analytics_artifacts(db: &AletheiaDB) -> Result<ArtifactLoad, StoreError> {
    let artifact_node_ids = db.scan_nodes_by_label(LABEL_ANALYTICS_ARTIFACT_MANIFEST);
    // Pre-allocate hash map based on known node count to eliminate runtime hashing reallocations.
    let mut artifacts = HashMap::new();
    let mut artifact_nodes = HashMap::new();

    for node_id in artifact_node_ids {
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
    HashMap<String, StoredStatementLine>,
    HashMap<String, NodeId>,
);

pub fn load_import_batches_and_records(
    db: &AletheiaDB,
    transaction_nodes: &HashMap<TransactionId, NodeId>,
) -> Result<ImportLoad, StoreError> {
    let (batches, batch_nodes) = load_import_batches(db)?;
    let records = load_import_records(db, &batches, &batch_nodes)?;
    ensure_no_orphan_import_records(db, &records)?;
    let (statement_lines, statement_line_nodes) =
        load_statement_lines(db, transaction_nodes, &records)?;
    Ok((
        batches,
        records,
        batch_nodes,
        statement_lines,
        statement_line_nodes,
    ))
}

type ImportBatchLoad = (HashMap<String, StoredImportBatch>, HashMap<String, NodeId>);

pub fn load_import_batches(db: &AletheiaDB) -> Result<ImportBatchLoad, StoreError> {
    let batch_node_ids = db.scan_nodes_by_label(LABEL_LEDGER_IMPORT_BATCH);
    // Pre-allocate hash map based on known batch count to eliminate runtime hashing reallocations.
    let mut batches = HashMap::new();
    let mut batch_nodes = HashMap::new();

    for node_id in batch_node_ids {
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

pub fn parse_import_batch_node(node: &Node, batch_id: &str) -> Result<StoredImportBatch, StoreError> {
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

pub fn load_import_records(
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

pub fn validate_import_record_edge_count(
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

pub fn parse_import_record_from_edge(
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

pub fn ensure_no_orphan_import_records(
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

type StatementLineLoad = (
    HashMap<String, StoredStatementLine>,
    HashMap<String, NodeId>,
);

#[allow(clippy::too_many_lines)]
pub fn load_statement_lines(
    db: &AletheiaDB,
    transaction_nodes: &HashMap<TransactionId, NodeId>,
    records: &HashMap<String, StoredImportRecord>,
) -> Result<StatementLineLoad, StoreError> {
    let line_node_ids = db.scan_nodes_by_label(LABEL_LEDGER_STATEMENT_LINE);
    // Pre-allocate hash map based on known line node count to eliminate runtime hashing reallocations.
    let mut lines = HashMap::new();
    let mut line_nodes = HashMap::new();
    let transaction_ids_by_node: HashMap<_, _> = transaction_nodes
        .iter()
        .map(|(txn_id, node_id)| (*node_id, txn_id.clone()))
        .collect();

    for node_id in line_node_ids {
        let node = db
            .get_node(node_id)
            .map_err(|err| map_load_error("unable to read LedgerStatementLine node", err))?;
        let line_id = required_node_string_property(&node, PROP_STATEMENT_LINE_ID)?;
        if lines.contains_key(&line_id) {
            return Err(StoreError::LoadFailed {
                message: format!("duplicate statement line id '{line_id}' in graph projection"),
            });
        }

        let batch_id = required_node_string_property(&node, PROP_IMPORT_BATCH_ID)?;
        let source_uri = required_node_string_property(&node, PROP_STATEMENT_SOURCE_URI)?;
        let statement_timestamp = required_node_string_property(&node, PROP_STATEMENT_TIMESTAMP)?;
        let memo = required_node_string_property(&node, PROP_STATEMENT_MEMO)?;
        let amount_cents = required_node_i64_property(&node, PROP_STATEMENT_AMOUNT_CENTS)?;
        let imported_txn_id = optional_node_string_property(&node, PROP_IMPORT_IMPORTED_TXN_ID)
            .filter(|value| !value.is_empty())
            .map(|value| TransactionId::new(&value));
        let imported_at = required_node_i64_property(&node, PROP_IMPORT_IMPORTED_AT_US)?.into();

        if let Some(txn_id) = &imported_txn_id {
            if !transaction_nodes.contains_key(txn_id) {
                return Err(StoreError::LoadFailed {
                    message: format!(
                        "statement line '{line_id}' references unknown transaction '{}'",
                        txn_id.as_str()
                    ),
                });
            }

            let evidences_edges = db.get_outgoing_edges_with_label(node_id, EDGE_EVIDENCES_TXN);
            if evidences_edges.len() != 1 {
                return Err(StoreError::LoadFailed {
                    message: format!(
                        "statement line '{line_id}' expected one EVIDENCES_TXN edge, found {}",
                        evidences_edges.len()
                    ),
                });
            }

            let edge = db
                .get_edge(evidences_edges[0])
                .map_err(|err| map_load_error("unable to read EVIDENCES_TXN edge", err))?;
            let Some(edge_txn_id) = transaction_ids_by_node.get(&edge.target) else {
                return Err(StoreError::LoadFailed {
                    message: format!(
                        "statement line '{line_id}' EVIDENCES_TXN edge targets unknown transaction node {}",
                        edge.target.as_u64()
                    ),
                });
            };
            if edge_txn_id != txn_id {
                return Err(StoreError::LoadFailed {
                    message: format!(
                        "statement line '{line_id}' imported_txn_id '{}' does not match EVIDENCES_TXN edge '{}'",
                        txn_id.as_str(),
                        edge_txn_id.as_str()
                    ),
                });
            }
        }

        let line = StoredStatementLine::new(
            &line_id,
            &batch_id,
            &source_uri,
            &statement_timestamp,
            &memo,
            amount_cents,
            imported_txn_id,
            imported_at,
        );
        lines.insert(line_id.clone(), line);
        line_nodes.insert(line_id, node.id);
    }

    let mut seen_line_ids = HashSet::new();
    for node_id in db.scan_nodes_by_label(LABEL_LEDGER_IMPORT_RECORD) {
        let node = db
            .get_node(node_id)
            .map_err(|err| map_load_error("unable to read LedgerImportRecord node", err))?;
        let content_hash_key = required_node_string_property(&node, PROP_IMPORT_CONTENT_HASH_KEY)?;
        if !records.contains_key(&content_hash_key) {
            continue;
        }

        for edge_id in db.get_outgoing_edges_with_label(node_id, EDGE_HAS_STATEMENT_LINE) {
            let edge = db
                .get_edge(edge_id)
                .map_err(|err| map_load_error("unable to read HAS_STATEMENT_LINE edge", err))?;
            let line_node = db
                .get_node(edge.target)
                .map_err(|err| map_load_error("unable to read statement line edge target", err))?;
            let line_id = required_node_string_property(&line_node, PROP_STATEMENT_LINE_ID)?;
            seen_line_ids.insert(line_id);
        }
    }

    for line_id in lines.keys() {
        if !seen_line_ids.contains(line_id) {
            return Err(StoreError::LoadFailed {
                message: format!(
                    "statement line '{line_id}' is missing HAS_STATEMENT_LINE edge from an import record"
                ),
            });
        }
    }

    Ok((lines, line_nodes))
}

type ReconciliationLoad = (
    HashMap<String, StoredReconciliationRun>,
    HashMap<String, NodeId>,
    HashMap<String, Vec<String>>,
);

#[allow(clippy::too_many_lines)]
pub fn load_reconciliation_runs(
    db: &AletheiaDB,
    transaction_nodes: &HashMap<TransactionId, NodeId>,
    statement_line_nodes: &HashMap<String, NodeId>,
) -> Result<ReconciliationLoad, StoreError> {
    let run_node_ids = db.scan_nodes_by_label(LABEL_LEDGER_RECONCILIATION_RUN);
    // Pre-allocate hash maps based on known run node count to eliminate runtime hashing reallocations.
    let mut runs = HashMap::new();
    let mut run_nodes = HashMap::new();
    let mut run_statement_line_ids = HashMap::new();
    let transaction_ids_by_node: HashMap<_, _> = transaction_nodes
        .iter()
        .map(|(txn_id, node_id)| (*node_id, txn_id.clone()))
        .collect();
    let statement_line_ids_by_node: HashMap<_, _> = statement_line_nodes
        .iter()
        .map(|(line_id, node_id)| (*node_id, line_id.clone()))
        .collect();

    for node_id in run_node_ids {
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
        let mut statement_line_ids = Vec::new();
        let mut seen_statement_line_ids = HashSet::new();
        for edge_id in db.get_outgoing_edges_with_label(node_id, EDGE_RECONCILES_STMT_LINE) {
            let edge = db
                .get_edge(edge_id)
                .map_err(|err| map_load_error("unable to read RECONCILES_STMT_LINE edge", err))?;
            let Some(line_id) = statement_line_ids_by_node.get(&edge.target) else {
                return Err(StoreError::LoadFailed {
                    message: format!(
                        "reconciliation run '{run_id}' points to unknown statement line node {}",
                        edge.target.as_u64()
                    ),
                });
            };
            if !seen_statement_line_ids.insert(line_id.clone()) {
                return Err(StoreError::LoadFailed {
                    message: format!(
                        "reconciliation run '{run_id}' has duplicate RECONCILES_STMT_LINE edge to '{line_id}'"
                    ),
                });
            }
            statement_line_ids.push(line_id.clone());
        }
        statement_line_ids.sort();
        runs.insert(run_id.clone(), run);
        run_statement_line_ids.insert(run_id.clone(), statement_line_ids);
        run_nodes.insert(run_id, node.id);
    }

    Ok((runs, run_nodes, run_statement_line_ids))
}

type MonthCloseLoad = (HashMap<String, StoredMonthClose>, HashMap<String, NodeId>);

#[allow(clippy::too_many_lines)]
pub fn load_month_closes(
    db: &AletheiaDB,
    reconciliation_run_nodes: &HashMap<String, NodeId>,
    analytics_artifact_nodes: &HashMap<String, NodeId>,
) -> Result<MonthCloseLoad, StoreError> {
    let run_ids_by_node: HashMap<_, _> = reconciliation_run_nodes
        .iter()
        .map(|(run_id, node_id)| (*node_id, run_id.clone()))
        .collect();
    let artifact_ids_by_node: HashMap<_, _> = analytics_artifact_nodes
        .iter()
        .map(|(artifact_id, node_id)| (*node_id, artifact_id.clone()))
        .collect();

    let close_node_ids = db.scan_nodes_by_label(LABEL_LEDGER_MONTH_CLOSE);
    // Pre-allocate hash maps based on known close node count to eliminate runtime hashing reallocations.
    let mut closes = HashMap::new();
    let mut close_nodes = HashMap::new();

    for node_id in close_node_ids {
        let node = db
            .get_node(node_id)
            .map_err(|err| map_load_error("unable to read LedgerMonthClose node", err))?;

        let close_id = required_node_string_property(&node, PROP_MONTH_CLOSE_ID)?;
        if closes.contains_key(&close_id) {
            return Err(StoreError::LoadFailed {
                message: format!("duplicate month close id '{close_id}' in graph projection"),
            });
        }

        let month_key = required_node_string_property(&node, PROP_MONTH_KEY)?;
        let checking_account =
            required_node_string_property(&node, PROP_RECONCILIATION_CHECKING_ACCOUNT)?;
        let reconciliation_run_id =
            required_node_string_property(&node, PROP_MONTH_CLOSE_RECONCILIATION_RUN_ID)?;
        let analytics_artifact_id =
            optional_node_string_property(&node, PROP_MONTH_CLOSE_ANALYTICS_ARTIFACT_ID)
                .filter(|value| !value.is_empty());
        let closed_at = required_node_i64_property(&node, PROP_MONTH_CLOSE_CLOSED_AT_US)?.into();

        let run_node = reconciliation_run_nodes
            .get(&reconciliation_run_id)
            .copied()
            .ok_or_else(|| StoreError::LoadFailed {
                message: format!(
                    "month close '{close_id}' references unknown reconciliation run '{reconciliation_run_id}'"
                ),
            })?;
        let has_run_edge = db
            .get_outgoing_edges_with_label(node_id, EDGE_CLOSES_RECONCILIATION_RUN)
            .into_iter()
            .any(|edge_id| {
                db.get_edge(edge_id)
                    .map(|edge| edge.target == run_node)
                    .unwrap_or(false)
            });
        if !has_run_edge {
            return Err(StoreError::LoadFailed {
                message: format!(
                    "month close '{close_id}' is missing CLOSES_RECONCILIATION_RUN edge to '{reconciliation_run_id}'"
                ),
            });
        }

        if let Some(artifact_id) = &analytics_artifact_id {
            let artifact_node = analytics_artifact_nodes
                .get(artifact_id)
                .copied()
                .ok_or_else(|| StoreError::LoadFailed {
                    message: format!(
                        "month close '{close_id}' references unknown analytics artifact '{artifact_id}'"
                    ),
                })?;
            let has_artifact_edge = db
                .get_outgoing_edges_with_label(node_id, EDGE_CLOSES_ANALYTICS_ARTIFACT)
                .into_iter()
                .any(|edge_id| {
                    db.get_edge(edge_id)
                        .map(|edge| edge.target == artifact_node)
                        .unwrap_or(false)
                });
            if !has_artifact_edge {
                return Err(StoreError::LoadFailed {
                    message: format!(
                        "month close '{close_id}' is missing CLOSES_ANALYTICS_ARTIFACT edge to '{artifact_id}'"
                    ),
                });
            }
        }

        for edge_id in db.get_outgoing_edges_with_label(node_id, EDGE_CLOSES_ANALYTICS_ARTIFACT) {
            let edge = db.get_edge(edge_id).map_err(|err| {
                map_load_error("unable to read CLOSES_ANALYTICS_ARTIFACT edge", err)
            })?;
            if !artifact_ids_by_node.contains_key(&edge.target) {
                return Err(StoreError::LoadFailed {
                    message: format!(
                        "month close '{close_id}' has CLOSES_ANALYTICS_ARTIFACT edge to unknown node {}",
                        edge.target.as_u64()
                    ),
                });
            }
        }
        for edge_id in db.get_outgoing_edges_with_label(node_id, EDGE_CLOSES_RECONCILIATION_RUN) {
            let edge = db.get_edge(edge_id).map_err(|err| {
                map_load_error("unable to read CLOSES_RECONCILIATION_RUN edge", err)
            })?;
            if !run_ids_by_node.contains_key(&edge.target) {
                return Err(StoreError::LoadFailed {
                    message: format!(
                        "month close '{close_id}' has CLOSES_RECONCILIATION_RUN edge to unknown node {}",
                        edge.target.as_u64()
                    ),
                });
            }
        }

        let close = StoredMonthClose::new(
            &close_id,
            &month_key,
            &checking_account,
            &reconciliation_run_id,
            analytics_artifact_id.as_deref(),
            closed_at,
        );
        closes.insert(close_id.clone(), close);
        close_nodes.insert(close_id, node.id);
    }

    Ok((closes, close_nodes))
}

pub fn required_node_string_property(node: &Node, key: &str) -> Result<String, StoreError> {
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

pub fn required_node_bool_flag_property(node: &Node, key: &str) -> Result<bool, StoreError> {
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

pub fn required_node_i64_property(node: &Node, key: &str) -> Result<i64, StoreError> {
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

pub fn optional_node_i64_property(node: &Node, key: &str) -> Option<i64> {
    node.get_property(key)
        .and_then(aletheiadb::PropertyValue::as_int)
}

pub fn optional_node_string_property(node: &Node, key: &str) -> Option<String> {
    node.get_property(key)
        .and_then(|value| value.as_str())
        .map(str::to_owned)
}

pub fn required_edge_i64_property(edge: &Edge, key: &str) -> Result<i64, StoreError> {
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

pub fn parse_posting(txn_id: &str, account: &str, amount_cents: i64) -> Result<Posting, StoreError> {
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
    Posting::credit(account, credit_amount).map_err(|e| map_load_error("posting credit", e))
}

pub fn map_load_error(context: &str, error: impl fmt::Display) -> StoreError {
    StoreError::LoadFailed {
        message: format!("{context}: {error}"),
    }
}

pub fn map_persist_error(context: &str, error: impl fmt::Display) -> StoreError {
    StoreError::PersistFailed {
        message: format!("{context}: {error}"),
    }
}

pub fn index_statement_lines_by_transaction(
    statement_lines: &HashMap<String, StoredStatementLine>,
) -> HashMap<TransactionId, Vec<String>> {
    let mut by_txn = HashMap::<TransactionId, Vec<String>>::new();
    for line in statement_lines.values() {
        if let Some(txn_id) = line.imported_txn_id().cloned() {
            by_txn
                .entry(txn_id)
                .or_default()
                .push(line.line_id().to_owned());
        }
    }

    for line_ids in by_txn.values_mut() {
        line_ids.sort();
        line_ids.dedup();
    }
    by_txn
}

pub fn collect_statement_line_ids_for_transactions(
    statement_line_ids_by_txn: &HashMap<TransactionId, Vec<String>>,
    txn_ids: &[TransactionId],
) -> Vec<String> {
    let mut line_ids = Vec::new();
    for txn_id in txn_ids {
        if let Some(ids) = statement_line_ids_by_txn.get(txn_id) {
            line_ids.extend(ids.iter().cloned());
        }
    }
    line_ids.sort();
    line_ids.dedup();
    line_ids
}

pub fn index_month_close_by_scope(
    month_closes: &HashMap<String, StoredMonthClose>,
) -> HashMap<BudgetTargetKey, String> {
    let mut by_scope = HashMap::new();
    for close in month_closes.values() {
        by_scope.insert(
            (
                close.month_key().to_owned(),
                close.checking_account().to_owned(),
            ),
            close.close_id().to_owned(),
        );
    }
    by_scope
}

pub fn infer_next_id<'a, I>(ids: I) -> u64
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

pub fn infer_next_artifact_id<'a, I>(ids: I) -> u64
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

pub fn infer_next_import_batch_id<'a, I>(ids: I) -> u64
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

pub fn infer_next_statement_line_id<'a, I>(ids: I) -> u64
where
    I: Iterator<Item = &'a String>,
{
    ids.filter_map(|id| {
        id.strip_prefix("stmt-line-")
            .and_then(|value| value.parse::<u64>().ok())
    })
    .max()
    .unwrap_or(0)
}

pub fn infer_next_reconciliation_run_id<'a, I>(ids: I) -> u64
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

pub fn infer_next_month_close_id<'a, I>(ids: I) -> u64
where
    I: Iterator<Item = &'a String>,
{
    ids.filter_map(|id| {
        id.strip_prefix("close-")
            .and_then(|value| value.parse::<u64>().ok())
    })
    .max()
    .unwrap_or(0)
}
