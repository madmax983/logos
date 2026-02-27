use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use aletheiadb::{AletheiaDB, AletheiaDBConfig, DurabilityMode, WalConfigBuilder, time};
use logos_core::{Correction, Posting, TransactionBuilder, TransactionId};
use logos_store_aletheia::AletheiaStore;

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
                .posting(Posting::credit("income:salary", 10_000)),
        )
        .expect("write");

    assert!(store.has_transaction(&id));
    assert_eq!(store.transaction_count(), 1);
}

#[test]
fn unbalanced_transaction_is_rejected_before_persistence() {
    let mut store = AletheiaStore::new();
    let err = store
        .write_transaction(
            TransactionBuilder::new("bad")
                .posting(Posting::debit("assets:checking", 10_000))
                .posting(Posting::credit("income:salary", 9_000)),
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
                .posting(Posting::credit("income:salary", 10_000)),
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
                    .posting(Posting::credit("income:salary", 10_000)),
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
                    .posting(Posting::credit("income:salary", 10_000)),
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
fn embedded_mapping_writes_transaction_and_posting_graph_entities() {
    let path = temp_store_path("mapping-transaction");
    {
        let mut store = AletheiaStore::open(&path).expect("open");
        store
            .write_transaction(
                TransactionBuilder::new("paycheck")
                    .posting(Posting::debit("assets:checking", 10_000))
                    .posting(Posting::credit("income:salary", 10_000)),
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
                    .posting(Posting::credit("income:salary", 10_000)),
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
                    .posting(Posting::credit("income:salary", 7_500)),
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
                    .posting(Posting::credit("income:salary", 10_000)),
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
