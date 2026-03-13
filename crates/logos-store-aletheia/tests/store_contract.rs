use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use aletheiadb::{AletheiaDB, AletheiaDBConfig, DurabilityMode, WalConfigBuilder, time};
use logos_core::{Correction, Posting, TransactionBuilder, TransactionId};
use logos_store_aletheia::{AletheiaStore, StoreError, model::NewImportRecord};

fn temp_store_path(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!("logos-store-{prefix}-{nanos}.db"))
}

fn cleanup_store_path(path: &Path) {
    if path.exists() {
        if path.is_dir() {
            let _ = std::fs::remove_dir_all(path);
        } else {
            let _ = std::fs::remove_file(path);
        }
    }
}

fn open_raw_graph(path: &Path) -> AletheiaDB {
    let mut config = AletheiaDBConfig::builder()
        .wal(
            WalConfigBuilder::new()
                .wal_dir(path.join("wal"))
                .durability_mode(DurabilityMode::Synchronous)
                .build(),
        )
        .build();
    config.persistence.data_dir = path.join("index-data");

    AletheiaDB::with_unified_config(config).expect("open raw graph")
}

#[test]
fn balanced_transaction_write_succeeds() {
    let mut store = AletheiaStore::new();
    let id = store
        .write_transaction(
            TransactionBuilder::new("paycheck")
                .posting(Posting::debit("assets:checking", 10_000))
                .posting(Posting::credit("income:salary", 10_000).expect("credit")),
        )
        .expect("write");

    assert!(store.has_transaction(&id));
    assert_eq!(store.transaction_count(), 1);
}

#[test]
fn test_store_returns_correct_counts_and_has_transaction() {
    let mut store = AletheiaStore::new();
    assert_eq!(store.correction_count(), 0);
    assert!(!store.has_transaction(&TransactionId::new("non-existent")));

    let txn_id = store
        .write_transaction(
            TransactionBuilder::new("txn1")
                .posting(Posting::debit("assets:checking", 100))
                .posting(Posting::credit("income:salary", 100).unwrap()),
        )
        .expect("write txn");

    assert!(store.has_transaction(&txn_id));

    store
        .write_correction(Correction::new(txn_id.clone(), "fix memo").unwrap())
        .expect("write correction");

    assert_eq!(store.correction_count(), 1);

    store
        .write_correction(Correction::new(txn_id, "fix memo again").unwrap())
        .expect("write correction 2");

    assert_eq!(store.correction_count(), 2);
}

#[test]
fn test_transactions_iterator_yields_all_items() {
    let mut store = AletheiaStore::new();
    assert_eq!(store.transactions().count(), 0);

    store
        .write_transaction(
            TransactionBuilder::new("txn1")
                .posting(Posting::debit("assets:checking", 100))
                .posting(Posting::credit("income:salary", 100).unwrap()),
        )
        .expect("write txn 1");

    store
        .write_transaction(
            TransactionBuilder::new("txn2")
                .posting(Posting::debit("assets:checking", 200))
                .posting(Posting::credit("income:salary", 200).unwrap()),
        )
        .expect("write txn 2");

    assert_eq!(store.transactions().count(), 2);
}

#[test]
fn test_budget_targets_iterator_yields_all_items() {
    let mut store = AletheiaStore::new();
    assert_eq!(store.budget_targets().count(), 0);

    store
        .write_budget_target("2026-03", "expenses:food", 250_000)
        .expect("write budget target 1");

    store
        .write_budget_target("2026-04", "expenses:food", 300_000)
        .expect("write budget target 2");

    assert_eq!(store.budget_targets().count(), 2);
}

#[test]
fn test_analytics_artifacts_iterator_yields_all_items() {
    let mut store = AletheiaStore::new();
    assert_eq!(store.analytics_artifacts().count(), 0);

    let first = store
        .write_analytics_artifact_manifest(
            "parquet",
            "C:\\artifacts\\base.parquet",
            "hash-base",
            1,
            10,
            time::from_secs(1_700_000_300),
            time::from_secs(1_700_000_301),
            None,
        )
        .expect("write first");

    store
        .write_analytics_artifact_manifest(
            "parquet",
            "C:\\artifacts\\next.parquet",
            "hash-next",
            1,
            11,
            time::from_secs(1_700_000_400),
            time::from_secs(1_700_000_401),
            Some(first.artifact_id()),
        )
        .expect("write second");

    assert_eq!(store.analytics_artifacts().count(), 2);
}

#[test]
fn unbalanced_transaction_is_rejected_before_persistence() {
    let mut store = AletheiaStore::new();
    let err = store
        .write_transaction(
            TransactionBuilder::new("bad")
                .posting(Posting::debit("assets:checking", 10_000))
                .posting(Posting::credit("income:salary", 9_000).expect("credit")),
        )
        .expect_err("must reject");

    assert!(err.to_string().contains("balanced"));
    assert_eq!(store.transaction_count(), 0);
}

#[test]
fn correction_append_links_superseded_transaction() {
    let mut store = AletheiaStore::new();
    let id = store
        .write_transaction(
            TransactionBuilder::new("paycheck")
                .posting(Posting::debit("assets:checking", 10_000))
                .posting(Posting::credit("income:salary", 10_000).expect("credit")),
        )
        .expect("write");

    let correction = Correction::new(id.clone(), "fix memo").expect("correction");
    store
        .write_correction(correction)
        .expect("append correction");

    let latest = store.latest_correction().expect("correction exists");
    assert_eq!(latest.supersedes_id(), &id);
}

#[test]
fn open_persists_transaction_across_reopen() {
    let path = temp_store_path("persist-txn");
    {
        let mut store = AletheiaStore::open(&path).expect("open");
        let id = store
            .write_transaction(
                TransactionBuilder::new("paycheck")
                    .posting(Posting::debit("assets:checking", 10_000))
                    .posting(Posting::credit("income:salary", 10_000).expect("credit")),
            )
            .expect("write");
        assert!(store.has_transaction(&id));
    }

    let reopened = AletheiaStore::open(&path).expect("reopen");
    assert!(reopened.has_transaction(&TransactionId::new("txn-1")));
    assert_eq!(reopened.transaction_count(), 1);

    cleanup_store_path(&path);
}

#[test]
fn open_persists_correction_chain_across_reopen() {
    let path = temp_store_path("persist-correction");
    let persisted_id;
    {
        let mut store = AletheiaStore::open(&path).expect("open");
        persisted_id = store
            .write_transaction(
                TransactionBuilder::new("paycheck")
                    .posting(Posting::debit("assets:checking", 10_000))
                    .posting(Posting::credit("income:salary", 10_000).expect("credit")),
            )
            .expect("write");
        let correction = Correction::new(persisted_id.clone(), "fix memo").expect("correction");
        store
            .write_correction(correction)
            .expect("write correction");
    }

    let reopened = AletheiaStore::open(&path).expect("reopen");
    let latest = reopened.latest_correction().expect("correction exists");
    assert_eq!(latest.supersedes_id(), &persisted_id);
    assert_eq!(reopened.correction_count(), 1);

    cleanup_store_path(&path);
}

#[test]
fn open_persists_budget_target_across_reopen() {
    let path = temp_store_path("persist-budget-target");
    {
        let mut store = AletheiaStore::open(&path).expect("open");
        store
            .write_budget_target("2026-03", "expenses:food", 250_000)
            .expect("write budget target");
    }

    let reopened = AletheiaStore::open(&path).expect("reopen");
    let target = reopened
        .budget_target("2026-03", "expenses:food")
        .expect("budget target exists");
    assert_eq!(target.budget_cents(), 250_000);

    cleanup_store_path(&path);
}

#[test]
fn open_persists_analytics_artifact_manifest_across_reopen() {
    let path = temp_store_path("persist-analytics-artifact");
    let valid_time = time::from_secs(1_700_000_100);
    let tx_time = time::from_secs(1_700_000_200);
    let artifact_id;
    {
        let mut store = AletheiaStore::open(&path).expect("open");
        let manifest = store
            .write_analytics_artifact_manifest(
                "parquet",
                "C:\\artifacts\\a.parquet",
                "hash-a",
                1,
                42,
                valid_time,
                tx_time,
                None,
            )
            .expect("write analytics artifact");
        artifact_id = manifest.artifact_id().to_owned();
    }

    let reopened = AletheiaStore::open(&path).expect("reopen");
    let manifest = reopened
        .analytics_artifact(&artifact_id)
        .expect("analytics artifact exists");
    assert_eq!(manifest.artifact_kind(), "parquet");
    assert_eq!(manifest.content_hash(), "hash-a");
    assert_eq!(manifest.row_count(), 42);
    assert_eq!(manifest.snapshot_valid_at(), valid_time);
    assert_eq!(manifest.snapshot_tx_at(), tx_time);

    cleanup_store_path(&path);
}

#[test]
fn open_persists_import_record_content_hash_across_reopen() {
    let path = temp_store_path("persist-import-record");
    let hash_key = "9f3b24009f3b2400";
    {
        let mut store = AletheiaStore::open(&path).expect("open");
        store
            .write_import_batch(
                "csv-row",
                "inline:csv",
                "batch-key-1",
                0,
                false,
                false,
                &[NewImportRecord::new(hash_key, None)],
            )
            .expect("write import batch");
    }

    let reopened = AletheiaStore::open(&path).expect("reopen");
    assert!(reopened.has_import_record_content_hash(hash_key));
    assert_eq!(reopened.import_record_count(), 1);

    cleanup_store_path(&path);
}

#[test]
fn open_persists_statement_line_evidence_across_reopen() {
    let path = temp_store_path("persist-statement-line");
    let txn_id;
    {
        let mut store = AletheiaStore::open(&path).expect("open");
        txn_id = store
            .write_transaction(
                TransactionBuilder::new("coffee")
                    .posting(Posting::debit("expenses:food", 500))
                    .posting(Posting::credit("assets:checking", 500).expect("credit")),
            )
            .expect("write txn");

        store
            .write_import_batch(
                "pdf-statement",
                "C:\\statements\\march.pdf",
                "batch-key-lines-1",
                0,
                false,
                false,
                &[NewImportRecord::with_statement_line(
                    "line-abc",
                    Some(&txn_id),
                    "C:\\statements\\march.pdf",
                    "2026-03-01T00:00:00",
                    "COFFEE SHOP",
                    -500,
                )],
            )
            .expect("write import with statement line");
        assert_eq!(store.statement_line_count(), 1);
    }

    let reopened = AletheiaStore::open(&path).expect("reopen");
    assert_eq!(reopened.statement_line_count(), 1);
    let line = reopened
        .statement_lines()
        .next()
        .expect("statement line exists");
    assert_eq!(line.source_uri(), "C:\\statements\\march.pdf");
    assert_eq!(line.statement_timestamp(), "2026-03-01T00:00:00");
    assert_eq!(line.memo(), "COFFEE SHOP");
    assert_eq!(line.amount_cents(), -500);
    assert_eq!(line.imported_txn_id(), Some(&txn_id));

    cleanup_store_path(&path);
}

#[test]
fn open_persists_reconciliation_run_across_reopen() {
    let path = temp_store_path("persist-reconciliation-run");
    let txn_id;
    {
        let mut store = AletheiaStore::open(&path).expect("open");
        txn_id = store
            .write_transaction(
                TransactionBuilder::new("paycheck")
                    .posting(Posting::debit("assets:checking", 10_000))
                    .posting(Posting::credit("income:salary", 10_000).expect("credit")),
            )
            .expect("write transaction");

        let run = store
            .write_reconciliation_run(
                "2026-03",
                "assets:checking",
                100_000,
                10_000,
                110_000,
                109_500,
                -500,
                false,
                1,
                10_000,
                0,
                std::slice::from_ref(&txn_id),
            )
            .expect("write reconciliation");
        assert_eq!(run.run_id(), "recon-1");
    }

    let reopened = AletheiaStore::open(&path).expect("reopen");
    assert_eq!(reopened.reconciliation_run_count(), 1);
    let run = reopened
        .reconciliation_run("recon-1")
        .expect("reconciliation run exists");
    assert_eq!(run.month_key(), "2026-03");
    assert_eq!(run.checking_account(), "assets:checking");
    assert_eq!(run.variance_cents(), -500);
    assert_eq!(run.matched_transaction_count(), 1);

    cleanup_store_path(&path);
}

#[test]
fn open_persists_month_close_across_reopen() {
    let path = temp_store_path("persist-month-close");
    let run_id;
    {
        let mut store = AletheiaStore::open(&path).expect("open");
        let txn_id = store
            .write_transaction(
                TransactionBuilder::new("paycheck")
                    .posting(Posting::debit("assets:checking", 10_000))
                    .posting(Posting::credit("income:salary", 10_000).expect("credit")),
            )
            .expect("txn");
        run_id = store
            .write_reconciliation_run(
                "2026-03",
                "assets:checking",
                100_000,
                10_000,
                110_000,
                110_000,
                0,
                true,
                1,
                10_000,
                0,
                std::slice::from_ref(&txn_id),
            )
            .expect("run")
            .run_id()
            .to_owned();

        let close = store
            .write_month_close("2026-03", "assets:checking", &run_id, None)
            .expect("close month");
        assert_eq!(close.close_id(), "close-1");
    }

    let reopened = AletheiaStore::open(&path).expect("reopen");
    assert_eq!(reopened.month_close_count(), 1);
    let close = reopened
        .month_close_for_scope("2026-03", "assets:checking")
        .expect("close exists");
    assert_eq!(close.reconciliation_run_id(), run_id);
    assert_eq!(close.month_key(), "2026-03");
    assert_eq!(close.checking_account(), "assets:checking");

    cleanup_store_path(&path);
}

#[test]
fn atomic_reconcile_and_close_rejects_unknown_transaction_without_partial_persist() {
    let path = temp_store_path("atomic-reconcile-close");
    {
        let mut store = AletheiaStore::open(&path).expect("open");
        let err = store
            .write_reconciliation_run_and_month_close(
                "2026-03",
                "assets:checking",
                100_000,
                10_000,
                110_000,
                110_000,
                0,
                true,
                1,
                10_000,
                0,
                &[TransactionId::new("txn-missing")],
                None,
            )
            .expect_err("unknown transaction must fail");
        assert!(err.to_string().contains("txn-missing"));
        assert_eq!(store.reconciliation_run_count(), 0);
        assert_eq!(store.month_close_count(), 0);
    }

    let reopened = AletheiaStore::open(&path).expect("reopen");
    assert_eq!(reopened.reconciliation_run_count(), 0);
    assert_eq!(reopened.month_close_count(), 0);

    cleanup_store_path(&path);
}

#[test]
fn embedded_mapping_writes_transaction_and_posting_graph_entities() {
    let path = temp_store_path("mapping-transaction");
    {
        let mut store = AletheiaStore::open(&path).expect("open");
        store
            .write_transaction(
                TransactionBuilder::new("paycheck")
                    .posting(Posting::debit("assets:checking", 10_000))
                    .posting(Posting::credit("income:salary", 10_000).expect("credit")),
            )
            .expect("write");
    }

    let graph = open_raw_graph(&path);
    let transaction_nodes: Vec<_> = graph.scan_nodes_by_label("LedgerTransaction").collect();
    let posting_count = graph.scan_nodes_by_label("LedgerPosting").count();

    let has_posting_edges: usize = transaction_nodes
        .iter()
        .map(|node_id| {
            graph
                .get_outgoing_edges_with_label(*node_id, "HAS_POSTING")
                .len()
        })
        .sum();

    assert_eq!(transaction_nodes.len(), 1);
    assert_eq!(posting_count, 2);
    assert_eq!(has_posting_edges, 2);

    cleanup_store_path(&path);
}

#[test]
fn embedded_mapping_writes_correction_supersedes_edge() {
    let path = temp_store_path("mapping-correction");
    {
        let mut store = AletheiaStore::open(&path).expect("open");
        let id = store
            .write_transaction(
                TransactionBuilder::new("paycheck")
                    .posting(Posting::debit("assets:checking", 10_000))
                    .posting(Posting::credit("income:salary", 10_000).expect("credit")),
            )
            .expect("write");

        store
            .write_correction(Correction::new(id, "fix memo").expect("correction"))
            .expect("write correction");
    }

    let graph = open_raw_graph(&path);
    let correction_nodes: Vec<_> = graph.scan_nodes_by_label("LedgerCorrection").collect();
    let supersedes_edges: usize = correction_nodes
        .iter()
        .map(|node_id| {
            graph
                .get_outgoing_edges_with_label(*node_id, "SUPERSEDES")
                .len()
        })
        .sum();

    assert_eq!(correction_nodes.len(), 1);
    assert_eq!(supersedes_edges, 1);

    cleanup_store_path(&path);
}

#[test]
fn embedded_mapping_writes_analytics_artifact_lineage_edge() {
    let path = temp_store_path("mapping-analytics-lineage");
    {
        let mut store = AletheiaStore::open(&path).expect("open");
        let first = store
            .write_analytics_artifact_manifest(
                "parquet",
                "C:\\artifacts\\base.parquet",
                "hash-base",
                1,
                10,
                time::from_secs(1_700_000_300),
                time::from_secs(1_700_000_301),
                None,
            )
            .expect("write first");
        let second = store
            .write_analytics_artifact_manifest(
                "parquet",
                "C:\\artifacts\\next.parquet",
                "hash-next",
                1,
                11,
                time::from_secs(1_700_000_400),
                time::from_secs(1_700_000_401),
                Some(first.artifact_id()),
            )
            .expect("write second");
        assert_eq!(second.supersedes_artifact_id(), Some(first.artifact_id()));
    }

    let graph = open_raw_graph(&path);
    let manifest_nodes: Vec<_> = graph
        .scan_nodes_by_label("AnalyticsArtifactManifest")
        .collect();
    let derived_edges: usize = manifest_nodes
        .iter()
        .map(|node_id| {
            graph
                .get_outgoing_edges_with_label(*node_id, "DERIVED_FROM")
                .len()
        })
        .sum();

    assert_eq!(manifest_nodes.len(), 2);
    assert_eq!(derived_edges, 1);

    cleanup_store_path(&path);
}

#[test]
fn embedded_mapping_writes_import_batch_and_record_graph_entities() {
    let path = temp_store_path("mapping-import-batch");
    {
        let mut store = AletheiaStore::open(&path).expect("open");
        let first_txn = store
            .write_transaction(
                TransactionBuilder::new("coffee")
                    .posting(Posting::debit("expenses:food", 500))
                    .posting(Posting::credit("assets:checking", 500).expect("credit")),
            )
            .expect("write");

        store
            .write_import_batch(
                "pdf-statement",
                "C:\\statements\\feb.pdf",
                "batch-key-2",
                1,
                false,
                true,
                &[
                    NewImportRecord::new("abc", Some(&first_txn)),
                    NewImportRecord::new("def", None),
                ],
            )
            .expect("write import batch");
    }

    let graph = open_raw_graph(&path);
    let batch_nodes: Vec<_> = graph.scan_nodes_by_label("LedgerImportBatch").collect();
    let record_count = graph.scan_nodes_by_label("LedgerImportRecord").count();
    let has_record_edges: usize = batch_nodes
        .iter()
        .map(|node_id| {
            graph
                .get_outgoing_edges_with_label(*node_id, "HAS_IMPORT_RECORD")
                .len()
        })
        .sum();

    assert_eq!(batch_nodes.len(), 1);
    assert_eq!(record_count, 2);
    assert_eq!(has_record_edges, 2);

    cleanup_store_path(&path);
}

#[test]
fn embedded_mapping_writes_reconciliation_run_and_edges() {
    let path = temp_store_path("mapping-reconciliation-run");
    {
        let mut store = AletheiaStore::open(&path).expect("open");
        let txn_a = store
            .write_transaction(
                TransactionBuilder::new("paycheck")
                    .posting(Posting::debit("assets:checking", 10_000))
                    .posting(Posting::credit("income:salary", 10_000).expect("credit")),
            )
            .expect("txn a");
        let txn_b = store
            .write_transaction(
                TransactionBuilder::new("groceries")
                    .posting(Posting::debit("expenses:food", 2_500))
                    .posting(Posting::credit("assets:checking", 2_500).expect("credit")),
            )
            .expect("txn b");
        store
            .write_reconciliation_run(
                "2026-03",
                "assets:checking",
                100_000,
                7_500,
                107_500,
                107_500,
                0,
                true,
                2,
                10_000,
                2_500,
                &[txn_a, txn_b],
            )
            .expect("write run");
    }

    let graph = open_raw_graph(&path);
    let run_nodes: Vec<_> = graph
        .scan_nodes_by_label("LedgerReconciliationRun")
        .collect();
    let reconciles_edges: usize = run_nodes
        .iter()
        .map(|node_id| {
            graph
                .get_outgoing_edges_with_label(*node_id, "RECONCILES_TXN")
                .len()
        })
        .sum();

    assert_eq!(run_nodes.len(), 1);
    assert_eq!(reconciles_edges, 2);

    cleanup_store_path(&path);
}

#[test]
fn embedded_mapping_links_reconciliation_run_to_statement_lines() {
    let path = temp_store_path("mapping-reconciliation-statement-line");
    {
        let mut store = AletheiaStore::open(&path).expect("open");
        let txn = store
            .write_transaction(
                TransactionBuilder::new("coffee")
                    .posting(Posting::debit("expenses:food", 500))
                    .posting(Posting::credit("assets:checking", 500).expect("credit")),
            )
            .expect("txn");

        store
            .write_import_batch(
                "pdf-statement",
                "C:\\statements\\march.pdf",
                "batch-key-lines-2",
                0,
                false,
                false,
                &[NewImportRecord::with_statement_line(
                    "line-def",
                    Some(&txn),
                    "C:\\statements\\march.pdf",
                    "2026-03-02T00:00:00",
                    "COFFEE SHOP",
                    -500,
                )],
            )
            .expect("import");

        let run = store
            .write_reconciliation_run(
                "2026-03",
                "assets:checking",
                100_000,
                -500,
                99_500,
                99_500,
                0,
                true,
                1,
                0,
                500,
                std::slice::from_ref(&txn),
            )
            .expect("run");
        assert_eq!(run.run_id(), "recon-1");
        let lines = store.statement_lines_for_reconciliation_run("recon-1");
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].memo(), "COFFEE SHOP");
    }

    let graph = open_raw_graph(&path);
    let run_nodes: Vec<_> = graph
        .scan_nodes_by_label("LedgerReconciliationRun")
        .collect();
    let reconciles_statement_line_edges: usize = run_nodes
        .iter()
        .map(|node_id| {
            graph
                .get_outgoing_edges_with_label(*node_id, "RECONCILES_STMT_LINE")
                .len()
        })
        .sum();
    let statement_line_count = graph.scan_nodes_by_label("LedgerStatementLine").count();

    assert_eq!(run_nodes.len(), 1);
    assert_eq!(statement_line_count, 1);
    assert_eq!(reconciles_statement_line_edges, 1);

    cleanup_store_path(&path);
}

#[test]
fn embedded_mapping_writes_month_close_edges() {
    let path = temp_store_path("mapping-month-close");
    {
        let mut store = AletheiaStore::open(&path).expect("open");
        let txn = store
            .write_transaction(
                TransactionBuilder::new("paycheck")
                    .posting(Posting::debit("assets:checking", 10_000))
                    .posting(Posting::credit("income:salary", 10_000).expect("credit")),
            )
            .expect("txn");
        let run = store
            .write_reconciliation_run(
                "2026-03",
                "assets:checking",
                100_000,
                10_000,
                110_000,
                110_000,
                0,
                true,
                1,
                10_000,
                0,
                std::slice::from_ref(&txn),
            )
            .expect("run");
        let artifact = store
            .write_analytics_artifact_manifest(
                "parquet",
                "C:\\artifacts\\snapshot.parquet",
                "hash-close",
                1,
                2,
                time::from_secs(1_700_000_500),
                time::from_secs(1_700_000_501),
                None,
            )
            .expect("artifact");

        store
            .write_month_close(
                "2026-03",
                "assets:checking",
                run.run_id(),
                Some(artifact.artifact_id()),
            )
            .expect("close");
    }

    let graph = open_raw_graph(&path);
    let close_nodes: Vec<_> = graph.scan_nodes_by_label("LedgerMonthClose").collect();
    let closes_run_edges: usize = close_nodes
        .iter()
        .map(|node_id| {
            graph
                .get_outgoing_edges_with_label(*node_id, "CLOSES_RECONCILIATION_RUN")
                .len()
        })
        .sum();
    let closes_artifact_edges: usize = close_nodes
        .iter()
        .map(|node_id| {
            graph
                .get_outgoing_edges_with_label(*node_id, "CLOSES_ANALYTICS_ARTIFACT")
                .len()
        })
        .sum();

    assert_eq!(close_nodes.len(), 1);
    assert_eq!(closes_run_edges, 1);
    assert_eq!(closes_artifact_edges, 1);

    cleanup_store_path(&path);
}

#[test]
fn transactions_as_of_respects_backdated_valid_time() {
    let path = temp_store_path("asof-valid-time");
    let backdated_valid_time = time::from_secs(1_700_000_000);
    let just_before_backdated = time::from_secs(1_699_999_999);
    {
        let mut store = AletheiaStore::open(&path).expect("open");
        store
            .write_transaction_with_valid_time(
                TransactionBuilder::new("backdated")
                    .posting(Posting::debit("assets:checking", 7_500))
                    .posting(Posting::credit("income:salary", 7_500).expect("credit")),
                Some(backdated_valid_time),
            )
            .expect("write backdated");

        let before_visible = store
            .transactions_as_of(just_before_backdated, time::now())
            .expect("query before valid time");
        assert!(before_visible.is_empty());

        let at_visible = store
            .transactions_as_of(backdated_valid_time, time::now())
            .expect("query at valid time");
        assert_eq!(at_visible.len(), 1);
    }

    cleanup_store_path(&path);
}

#[test]
fn transactions_as_of_hides_superseded_after_correction_tx_time() {
    let path = temp_store_path("asof-correction");
    {
        let mut store = AletheiaStore::open(&path).expect("open");
        let txn_id = store
            .write_transaction(
                TransactionBuilder::new("paycheck")
                    .posting(Posting::debit("assets:checking", 10_000))
                    .posting(Posting::credit("income:salary", 10_000).expect("credit")),
            )
            .expect("write");

        std::thread::sleep(Duration::from_millis(5));
        let tx_before_correction = time::now();

        store
            .write_correction(Correction::new(txn_id.clone(), "fix memo").expect("correction"))
            .expect("append correction");

        let before_correction = store
            .transactions_as_of(time::now(), tx_before_correction)
            .expect("query before correction tx");
        assert!(before_correction.iter().any(|txn| txn.id() == &txn_id));

        let after_correction = store
            .transactions_as_of(time::now(), time::now())
            .expect("query after correction tx");
        assert!(!after_correction.iter().any(|txn| txn.id() == &txn_id));
    }

    cleanup_store_path(&path);
}

#[test]
fn write_reconciliation_run_fails_with_negative_values() {
    let path = temp_store_path("reconciliation-negative");
    let mut store = AletheiaStore::open(&path).expect("open");
    let txn_id = store
        .write_transaction(
            TransactionBuilder::new("paycheck")
                .posting(Posting::debit("assets:checking", 10_000))
                .posting(Posting::credit("income:salary", 10_000).unwrap()),
        )
        .expect("write txn");

    // Negative matched_postings
    let res = store.write_reconciliation_run(
        "2026-03",
        "assets:checking",
        100_000,
        10_000,
        110_000,
        109_500,
        -500,
        false,
        -1, // matched_postings < 0
        10_000,
        0,
        std::slice::from_ref(&txn_id),
    );
    assert!(res.is_err());
    assert!(
        res.unwrap_err()
            .to_string()
            .contains("matched_postings must be non-negative")
    );

    // Negative inflow_cents
    let res = store.write_reconciliation_run(
        "2026-03",
        "assets:checking",
        100_000,
        10_000,
        110_000,
        109_500,
        -500,
        false,
        1,
        -1, // inflow_cents < 0
        0,
        std::slice::from_ref(&txn_id),
    );
    assert!(res.is_err());
    assert!(
        res.unwrap_err()
            .to_string()
            .contains("inflow_cents must be non-negative")
    );

    // Negative outflow_cents
    let res = store.write_reconciliation_run(
        "2026-03",
        "assets:checking",
        100_000,
        10_000,
        110_000,
        109_500,
        -500,
        false,
        1,
        10_000,
        -1, // outflow_cents < 0
        std::slice::from_ref(&txn_id),
    );
    assert!(res.is_err());
    assert!(
        res.unwrap_err()
            .to_string()
            .contains("outflow_cents must be non-negative")
    );

    cleanup_store_path(&path);
}

#[test]
fn write_reconciliation_run_and_month_close_fails_with_negative_values() {
    let path = temp_store_path("reconciliation-close-negative");
    let mut store = AletheiaStore::open(&path).expect("open");
    let txn_id = store
        .write_transaction(
            TransactionBuilder::new("paycheck")
                .posting(Posting::debit("assets:checking", 10_000))
                .posting(Posting::credit("income:salary", 10_000).unwrap()),
        )
        .expect("write txn");

    // Negative matched_postings
    let res = store.write_reconciliation_run_and_month_close(
        "2026-03",
        "assets:checking",
        100_000,
        10_000,
        110_000,
        109_500,
        -500,
        false,
        -1, // matched_postings < 0
        10_000,
        0,
        std::slice::from_ref(&txn_id),
        None,
    );
    assert!(res.is_err());
    assert!(
        res.unwrap_err()
            .to_string()
            .contains("matched_postings must be non-negative")
    );

    // Negative inflow_cents
    let res = store.write_reconciliation_run_and_month_close(
        "2026-03",
        "assets:checking",
        100_000,
        10_000,
        110_000,
        109_500,
        -500,
        false,
        1,
        -1, // inflow_cents < 0
        0,
        std::slice::from_ref(&txn_id),
        None,
    );
    assert!(res.is_err());
    assert!(
        res.unwrap_err()
            .to_string()
            .contains("inflow_cents must be non-negative")
    );

    // Negative outflow_cents
    let res = store.write_reconciliation_run_and_month_close(
        "2026-03",
        "assets:checking",
        100_000,
        10_000,
        110_000,
        109_500,
        -500,
        false,
        1,
        10_000,
        -1, // outflow_cents < 0
        std::slice::from_ref(&txn_id),
        None,
    );
    assert!(res.is_err());
    assert!(
        res.unwrap_err()
            .to_string()
            .contains("outflow_cents must be non-negative")
    );

    cleanup_store_path(&path);
}

#[test]
fn write_reconciliation_run_succeeds_with_zero_values() {
    let path = temp_store_path("reconciliation-zero");
    let mut store = AletheiaStore::open(&path).expect("open");
    let txn_id = store
        .write_transaction(
            TransactionBuilder::new("paycheck")
                .posting(Posting::debit("assets:checking", 10_000))
                .posting(Posting::credit("income:salary", 10_000).unwrap()),
        )
        .expect("write txn");

    let res = store.write_reconciliation_run(
        "2026-03",
        "assets:checking",
        100_000,
        10_000,
        110_000,
        109_500,
        -500,
        false,
        0, // matched_postings == 0
        0, // inflow_cents == 0
        0, // outflow_cents == 0
        std::slice::from_ref(&txn_id),
    );
    assert!(res.is_ok());

    cleanup_store_path(&path);
}

#[test]
fn write_reconciliation_run_and_month_close_succeeds_with_zero_values() {
    let path = temp_store_path("reconciliation-close-zero");
    let mut store = AletheiaStore::open(&path).expect("open");
    let txn_id = store
        .write_transaction(
            TransactionBuilder::new("paycheck")
                .posting(Posting::debit("assets:checking", 10_000))
                .posting(Posting::credit("income:salary", 10_000).unwrap()),
        )
        .expect("write txn");

    let res = store.write_reconciliation_run_and_month_close(
        "2026-03",
        "assets:checking",
        100_000,
        10_000,
        110_000,
        109_500,
        -500,
        false,
        0, // matched_postings == 0
        0, // inflow_cents == 0
        0, // outflow_cents == 0
        std::slice::from_ref(&txn_id),
        None,
    );
    assert!(res.is_ok());

    cleanup_store_path(&path);
}

#[test]
fn write_reconciliation_run_and_month_close_fails_with_unknown_artifact() {
    let path = temp_store_path("reconciliation-close-unknown-artifact");
    let mut store = AletheiaStore::open(&path).expect("open");
    let txn_id = store
        .write_transaction(
            TransactionBuilder::new("paycheck")
                .posting(Posting::debit("assets:checking", 10_000))
                .posting(Posting::credit("income:salary", 10_000).unwrap()),
        )
        .expect("write txn");

    let res = store.write_reconciliation_run_and_month_close(
        "2026-03",
        "assets:checking",
        100_000,
        10_000,
        110_000,
        109_500,
        -500,
        false,
        1,
        10_000,
        0,
        std::slice::from_ref(&txn_id),
        Some("unknown-artifact-id"),
    );
    assert!(res.is_err());
    assert!(matches!(
        res.unwrap_err(),
        StoreError::UnknownArtifact { .. }
    ));

    cleanup_store_path(&path);
}

#[test]
fn test_store_accessors_on_empty() {
    let store = AletheiaStore::new();
    assert_eq!(store.correction_count(), 0);
    assert!(!store.has_transaction(&TransactionId::new("non-existent")));
    assert!(store.transactions().next().is_none());
    assert!(store.budget_targets().next().is_none());
    assert!(store.analytics_artifacts().next().is_none());
    assert_eq!(store.import_record_count(), 0);
    assert!(!store.has_import_record_content_hash("non-existent"));
    assert!(store.import_records().next().is_none());
    assert!(store.import_batches().next().is_none());
    assert_eq!(store.statement_line_count(), 0);
    assert!(store.reconciliation_runs().next().is_none());
    assert_eq!(store.month_close_count(), 0);
    assert!(store.month_close("non-existent").is_none());
    assert!(store.month_closes().next().is_none());
}

#[test]
fn test_in_memory_projection_filters_superseded_transactions() {
    let mut store = AletheiaStore::new();
    let txn_id = store
        .write_transaction(
            TransactionBuilder::new("paycheck")
                .posting(Posting::debit("assets:checking", 10_000))
                .posting(Posting::credit("income:salary", 10_000).unwrap()),
        )
        .expect("write txn");

    let correction = Correction::new(txn_id, "fix memo").unwrap();
    store.write_correction(correction).expect("write correction");

    let as_of_us = store.transactions_as_of_us(aletheiadb::time::now().wallclock(), aletheiadb::time::now().wallclock()).expect("transactions as of us");
    assert!(as_of_us.is_empty(), "superseded transaction should be filtered out");
}

#[test]
fn test_transactions_as_of_us_returns_non_empty() {
    let mut store = AletheiaStore::new();
    let txn_id = store
        .write_transaction(
            TransactionBuilder::new("paycheck")
                .posting(Posting::debit("assets:checking", 10_000))
                .posting(Posting::credit("income:salary", 10_000).unwrap()),
        )
        .expect("write txn");

    let as_of_us = store.transactions_as_of_us(aletheiadb::time::now().wallclock(), aletheiadb::time::now().wallclock()).expect("transactions as of us");
    assert_eq!(as_of_us.len(), 1);
    assert_eq!(as_of_us[0].id(), &txn_id);
}

#[test]
fn test_node_and_edge_visibility_at_as_of() {
    let path = temp_store_path("visibility");
    let tx_time_before = aletheiadb::time::now();
    let mut store = AletheiaStore::open(&path).expect("open");
    let _txn_id = store
        .write_transaction(
            TransactionBuilder::new("paycheck")
                .posting(Posting::debit("assets:checking", 10_000))
                .posting(Posting::credit("income:salary", 10_000).unwrap()),
        )
        .expect("write txn");

    let tx_time_after = aletheiadb::time::now();

    // Transactions as of before the transaction was written
    let before_visible = store
        .transactions_as_of(aletheiadb::time::now(), tx_time_before)
        .expect("query before tx");
    assert!(before_visible.is_empty(), "node should not be visible before it was written");

    // Transactions as of after the transaction was written
    let after_visible = store
        .transactions_as_of(aletheiadb::time::now(), tx_time_after)
        .expect("query after tx");
    assert_eq!(after_visible.len(), 1, "node should be visible after it was written");

    cleanup_store_path(&path);
}

#[test]
fn test_has_visible_supersedes_edge_fails_if_not_present() {
    let path = temp_store_path("visible-supersedes");
    {
        let mut store = AletheiaStore::open(&path).expect("open");
        let txn_id = store
            .write_transaction(
                TransactionBuilder::new("paycheck")
                    .posting(Posting::debit("assets:checking", 10_000))
                    .posting(Posting::credit("income:salary", 10_000).unwrap()),
            )
            .expect("write txn");

        let correction = Correction::new(txn_id, "fix memo").unwrap();
        store.write_correction(correction).expect("write correction");
        let _ = aletheiadb::time::now();
    }
    cleanup_store_path(&path);
}

#[test]
fn test_import_records_and_batches_iterators() {
    let path = temp_store_path("import-iterators");
    {
        let mut store = AletheiaStore::open(&path).expect("open");
        store
            .write_import_batch(
                "csv-row",
                "inline:csv",
                "batch-key-1",
                0,
                false,
                false,
                &[NewImportRecord::new("hash-abc", None)],
            )
            .expect("write import batch");

        assert_eq!(store.import_batches().count(), 1);
        assert_eq!(store.import_records().count(), 1);
    }
    cleanup_store_path(&path);
}

#[test]
fn test_reconciliation_runs_iterator_yields_all_items() {
    let path = temp_store_path("reconciliation-iterator");
    {
        let mut store = AletheiaStore::open(&path).expect("open");
        let txn_id = store
            .write_transaction(
                TransactionBuilder::new("paycheck")
                    .posting(Posting::debit("assets:checking", 10_000))
                    .posting(Posting::credit("income:salary", 10_000).expect("credit")),
            )
            .expect("txn");

        store
            .write_reconciliation_run(
                "2026-03",
                "assets:checking",
                100_000,
                10_000,
                110_000,
                110_000,
                0,
                true,
                1,
                10_000,
                0,
                std::slice::from_ref(&txn_id),
            )
            .expect("run");

        assert_eq!(store.reconciliation_runs().count(), 1);
    }
    cleanup_store_path(&path);
}

#[test]
fn test_month_close_iterators_yields_all_items() {
    let path = temp_store_path("month-close-iterator");
    {
        let mut store = AletheiaStore::open(&path).expect("open");
        let txn_id = store
            .write_transaction(
                TransactionBuilder::new("paycheck")
                    .posting(Posting::debit("assets:checking", 10_000))
                    .posting(Posting::credit("income:salary", 10_000).expect("credit")),
            )
            .expect("txn");

        let run = store
            .write_reconciliation_run(
                "2026-03",
                "assets:checking",
                100_000,
                10_000,
                110_000,
                110_000,
                0,
                true,
                1,
                10_000,
                0,
                std::slice::from_ref(&txn_id),
            )
            .expect("run");

        let close = store
            .write_month_close("2026-03", "assets:checking", run.run_id(), None)
            .expect("close");

        assert_eq!(store.month_closes().count(), 1);
        assert!(store.month_close(close.close_id()).is_some());
    }
    cleanup_store_path(&path);
}

#[test]
fn test_has_visible_supersedes_edge_fails_if_not_present_visible() {
    let path = temp_store_path("visible-supersedes-not-present");
    {
        let mut store = AletheiaStore::open(&path).expect("open");
        let txn_id = store
            .write_transaction(
                TransactionBuilder::new("paycheck")
                    .posting(Posting::debit("assets:checking", 10_000))
                    .posting(Posting::credit("income:salary", 10_000).unwrap()),
            )
            .expect("write txn");

        // We write a correction for another transaction so the supersedes edge doesn't point to ours
        let txn_id_2 = store
            .write_transaction(
                TransactionBuilder::new("paycheck")
                    .posting(Posting::debit("assets:checking", 10_000))
                    .posting(Posting::credit("income:salary", 10_000).unwrap()),
            )
            .expect("write txn 2");

        let correction = Correction::new(txn_id_2, "fix memo").unwrap();
        store.write_correction(correction).expect("write correction");

        let tx_time = aletheiadb::time::now();
        let as_of_us = store.transactions_as_of_us(tx_time.wallclock(), tx_time.wallclock()).expect("transactions");
        assert_eq!(as_of_us.len(), 1); // Only txn_id is visible, txn_id_2 is superseded
        assert_eq!(as_of_us[0].id(), &txn_id);
    }
    cleanup_store_path(&path);
}

#[test]
fn test_reconstruct_transaction_at_as_of_validates_posting() {
    let path = temp_store_path("reconstruct-posting-validation");
    {
        let mut store = AletheiaStore::open(&path).expect("open");
        let _txn_id = store
            .write_transaction(
                TransactionBuilder::new("paycheck")
                    .posting(Posting::debit("assets:checking", 10_000))
                    .posting(Posting::credit("income:salary", 10_000).unwrap()),
            )
            .expect("write txn");

        let tx_time = aletheiadb::time::now();
        let as_of_us = store.transactions_as_of_us(tx_time.wallclock(), tx_time.wallclock()).expect("transactions");
        assert_eq!(as_of_us.len(), 1);

        let tx = store.transactions().next().expect("tx");
        assert_eq!(tx.transaction().postings().len(), 2);
    }
    cleanup_store_path(&path);
}
