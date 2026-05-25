use super::*;
use logos_core::{Correction, TransactionBuilder, TransactionId};

struct DummyStore;
impl LedgerStore for DummyStore {}

#[test]
fn test_default_transactions_as_of_us() {
    assert!(matches!(
        DummyStore.transactions_as_of_us(100, 100),
        Err(StoreError::LoadFailed { .. })
    ));
}

#[test]
fn test_default_write_transaction() {
    let tx = TransactionBuilder::new("test");
    assert!(matches!(
        DummyStore.write_transaction(tx),
        Err(StoreError::PersistFailed { .. })
    ));
}

#[test]
fn test_default_write_transaction_with_valid_time() {
    let tx = TransactionBuilder::new("test");
    assert!(matches!(
        DummyStore.write_transaction_with_valid_time(tx, None),
        Err(StoreError::PersistFailed { .. })
    ));
}

#[test]
fn test_default_write_correction() {
    assert!(matches!(
        DummyStore.write_correction(
            Correction::new(TransactionId::new("tx-1").unwrap(), "test").unwrap()
        ),
        Err(StoreError::PersistFailed { .. })
    ));
}

#[test]
#[should_panic(expected = "LedgerStore::has_transaction is not implemented")]
fn test_default_has_transaction() {
    let id = TransactionId::new("tx-1").unwrap();
    let _ = DummyStore.has_transaction(&id);
}

#[test]
fn test_default_write_import_batch() {
    assert!(matches!(
        DummyStore.write_import_batch("kind", "uri", "key", 0, false, false, &[]),
        Err(StoreError::PersistFailed { .. })
    ));
}

#[test]
fn test_default_write_budget_target() {
    assert!(matches!(
        DummyStore.write_budget_target("2024-01", "Expenses", 100),
        Err(StoreError::PersistFailed { .. })
    ));
}

#[test]
fn test_default_write_reconciliation_run_and_month_close() {
    assert!(matches!(
        DummyStore.write_reconciliation_run_and_month_close(
            "2024-01",
            "c",
            1,
            2,
            3,
            4,
            5,
            true,
            6,
            7,
            8,
            &[],
            None
        ),
        Err(StoreError::PersistFailed { .. })
    ));
}

#[test]
fn test_default_write_fetch_run() {
    assert!(matches!(
        DummyStore.write_fetch_run(
            "s",
            "i",
            "m",
            "a",
            crate::model::StoredFetchRunStatus::Failed,
            None,
            None,
            None,
            None,
            None
        ),
        Err(StoreError::PersistFailed { .. })
    ));
}

#[test]
fn test_default_write_analytics_artifact_manifest() {
    assert!(matches!(
        DummyStore.write_analytics_artifact_manifest("id", "type", "ver", 1, 2, 3, 4, None),
        Err(StoreError::PersistFailed { .. })
    ));
}

#[test]
fn test_default_write_analytics_artifact_manifest_us() {
    assert!(matches!(
        DummyStore.write_analytics_artifact_manifest_us("id", "type", "ver", 1, 2, 3, 4, None),
        Err(StoreError::PersistFailed { .. })
    ));
}

#[test]
fn test_default_write_month_close() {
    assert!(matches!(
        DummyStore.write_month_close("c", "2024-01", "a", None),
        Err(StoreError::PersistFailed { .. })
    ));
}

#[test]
fn test_default_write_reconciliation_run() {
    assert!(matches!(
        DummyStore.write_reconciliation_run("2024-01", "a", 1, 2, 3, 4, 5, true, 6, 7, 8, &[]),
        Err(StoreError::PersistFailed { .. })
    ));
}

#[test]
#[should_panic(expected = "LedgerStore::statement_lines is not implemented")]
fn test_default_statement_lines() {
    let _ = DummyStore.statement_lines();
}

#[test]
#[should_panic(expected = "LedgerStore::import_records is not implemented")]
fn test_default_import_records() {
    let _ = DummyStore.import_records();
}

#[test]
#[should_panic(expected = "LedgerStore::fetch_runs is not implemented")]
fn test_default_fetch_runs() {
    let _ = DummyStore.fetch_runs();
}

#[test]
#[should_panic(expected = "LedgerStore::reconciliation_runs is not implemented")]
fn test_default_reconciliation_runs() {
    let _ = DummyStore.reconciliation_runs();
}

#[test]
#[should_panic(expected = "LedgerStore::month_closes is not implemented")]
fn test_default_month_closes() {
    let _ = DummyStore.month_closes();
}

#[test]
#[should_panic(expected = "LedgerStore::import_batches is not implemented")]
fn test_default_import_batches() {
    let _ = DummyStore.import_batches();
}

#[test]
#[should_panic(expected = "LedgerStore::budget_targets is not implemented")]
fn test_default_budget_targets() {
    let _ = DummyStore.budget_targets();
}

#[test]
#[should_panic(expected = "LedgerStore::analytics_artifacts is not implemented")]
fn test_default_analytics_artifacts() {
    let _ = DummyStore.analytics_artifacts();
}

#[test]
#[should_panic(expected = "LedgerStore::transaction_count is not implemented")]
fn test_default_transaction_count() {
    let _ = DummyStore.transaction_count();
}

#[test]
#[should_panic(expected = "LedgerStore::correction_count is not implemented")]
fn test_default_correction_count() {
    let _ = DummyStore.correction_count();
}

#[test]
#[should_panic(expected = "LedgerStore::latest_correction is not implemented")]
fn test_default_latest_correction() {
    let _ = DummyStore.latest_correction();
}

#[test]
#[should_panic(expected = "LedgerStore::transactions is not implemented")]
fn test_default_transactions() {
    let _ = DummyStore.transactions();
}

#[test]
#[should_panic(expected = "LedgerStore::budget_target is not implemented")]
fn test_default_budget_target() {
    let _ = DummyStore.budget_target("a", "b");
}

#[test]
#[should_panic(expected = "LedgerStore::analytics_artifact is not implemented")]
fn test_default_analytics_artifact() {
    let _ = DummyStore.analytics_artifact("a");
}

#[test]
#[should_panic(expected = "LedgerStore::import_record_count is not implemented")]
fn test_default_import_record_count() {
    let _ = DummyStore.import_record_count();
}

#[test]
#[should_panic(expected = "LedgerStore::has_import_record_content_hash is not implemented")]
fn test_default_has_import_record_content_hash() {
    let _ = DummyStore.has_import_record_content_hash("a");
}

#[test]
#[should_panic(expected = "LedgerStore::statement_line_count is not implemented")]
fn test_default_statement_line_count() {
    let _ = DummyStore.statement_line_count();
}

#[test]
#[should_panic(expected = "LedgerStore::fetch_run_count is not implemented")]
fn test_default_fetch_run_count() {
    let _ = DummyStore.fetch_run_count();
}

#[test]
#[should_panic(expected = "LedgerStore::fetch_run is not implemented")]
fn test_default_fetch_run() {
    let _ = DummyStore.fetch_run("a");
}

#[test]
#[should_panic(expected = "LedgerStore::statement_lines_for_reconciliation_run is not implemented")]
fn test_default_statement_lines_for_reconciliation_run() {
    let _ = DummyStore.statement_lines_for_reconciliation_run("a");
}

#[test]
#[should_panic(expected = "LedgerStore::reconciliation_run_count is not implemented")]
fn test_default_reconciliation_run_count() {
    let _ = DummyStore.reconciliation_run_count();
}

#[test]
#[should_panic(expected = "LedgerStore::reconciliation_run is not implemented")]
fn test_default_reconciliation_run() {
    let _ = DummyStore.reconciliation_run("a");
}

#[test]
#[should_panic(expected = "LedgerStore::month_close_count is not implemented")]
fn test_default_month_close_count() {
    let _ = DummyStore.month_close_count();
}

#[test]
#[should_panic(expected = "LedgerStore::month_close is not implemented")]
fn test_default_month_close() {
    let _ = DummyStore.month_close("a");
}

#[test]
#[should_panic(expected = "LedgerStore::month_close_for_scope is not implemented")]
fn test_default_month_close_for_scope() {
    let _ = DummyStore.month_close_for_scope("a", "b");
}
