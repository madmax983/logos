use logos_core::{Correction, TransactionBuilder, TransactionId};
use logos_store::{LedgerStore, StoreError, StoredFetchRunStatus};

struct DummyStore;
impl LedgerStore for DummyStore {}

// Helper macro to generate tests that expect panics
macro_rules! test_panic {
    ($name:ident, $method:ident, $expected:expr) => {
        #[test]
        #[should_panic(expected = $expected)]
        fn $name() {
            let store = DummyStore;
            store.$method();
        }
    };
    ($name:ident, $method:ident, $arg1:expr, $expected:expr) => {
        #[test]
        #[should_panic(expected = $expected)]
        fn $name() {
            let store = DummyStore;
            store.$method($arg1);
        }
    };
    ($name:ident, $method:ident, $arg1:expr, $arg2:expr, $expected:expr) => {
        #[test]
        #[should_panic(expected = $expected)]
        fn $name() {
            let store = DummyStore;
            store.$method($arg1, $arg2);
        }
    };
}

// Helper macro to generate tests that expect StoreError
macro_rules! test_error {
    ($name:ident, $method:ident, $expected_variant:path, $expected_msg:expr) => {
        #[test]
        fn $name() {
            #[allow(unused_mut)]
            let mut store = DummyStore;
            let result = store.$method();
            match result {
                Err($expected_variant { message }) => {
                    assert_eq!(message, $expected_msg);
                }
                _ => panic!("Expected error not returned"),
            }
        }
    };
    ($name:ident, $method:ident, $arg1:expr, $expected_variant:path, $expected_msg:expr) => {
        #[test]
        fn $name() {
            #[allow(unused_mut)]
            let mut store = DummyStore;
            let result = store.$method($arg1);
            match result {
                Err($expected_variant { message }) => {
                    assert_eq!(message, $expected_msg);
                }
                _ => panic!("Expected error not returned"),
            }
        }
    };
    ($name:ident, $method:ident, $arg1:expr, $arg2:expr, $expected_variant:path, $expected_msg:expr) => {
        #[test]
        fn $name() {
            #[allow(unused_mut)]
            let mut store = DummyStore;
            let result = store.$method($arg1, $arg2);
            match result {
                Err($expected_variant { message }) => {
                    assert_eq!(message, $expected_msg);
                }
                _ => panic!("Expected error not returned"),
            }
        }
    };
    ($name:ident, $method:ident, $arg1:expr, $arg2:expr, $arg3:expr, $expected_variant:path, $expected_msg:expr) => {
        #[test]
        fn $name() {
            #[allow(unused_mut)]
            let mut store = DummyStore;
            let result = store.$method($arg1, $arg2, $arg3);
            match result {
                Err($expected_variant { message }) => {
                    assert_eq!(message, $expected_msg);
                }
                _ => panic!("Expected error not returned"),
            }
        }
    };
}

test_panic!(test_transaction_count, transaction_count, "LedgerStore::transaction_count is not implemented");
test_panic!(test_correction_count, correction_count, "LedgerStore::correction_count is not implemented");
#[test]
#[should_panic(expected = "LedgerStore::has_transaction is not implemented")]
fn test_has_transaction() {
    let store = DummyStore;
    let tx_id = TransactionId::new("tx-123").unwrap();
    store.has_transaction(&tx_id);
}
test_panic!(test_latest_correction, latest_correction, "LedgerStore::latest_correction is not implemented");
test_panic!(test_transactions, transactions, "LedgerStore::transactions is not implemented");
test_panic!(test_budget_target, budget_target, "2024-01", "Expenses", "LedgerStore::budget_target is not implemented");
test_panic!(test_budget_targets, budget_targets, "LedgerStore::budget_targets is not implemented");
test_panic!(test_analytics_artifact, analytics_artifact, "art-1", "LedgerStore::analytics_artifact is not implemented");
test_panic!(test_analytics_artifacts, analytics_artifacts, "LedgerStore::analytics_artifacts is not implemented");
test_panic!(test_import_record_count, import_record_count, "LedgerStore::import_record_count is not implemented");
test_panic!(test_has_import_record_content_hash, has_import_record_content_hash, "hash-1", "LedgerStore::has_import_record_content_hash is not implemented");
test_panic!(test_import_records, import_records, "LedgerStore::import_records is not implemented");
test_panic!(test_import_batches, import_batches, "LedgerStore::import_batches is not implemented");
test_panic!(test_statement_line_count, statement_line_count, "LedgerStore::statement_line_count is not implemented");
test_panic!(test_statement_lines, statement_lines, "LedgerStore::statement_lines is not implemented");
test_panic!(test_fetch_run_count, fetch_run_count, "LedgerStore::fetch_run_count is not implemented");
test_panic!(test_fetch_run, fetch_run, "run-1", "LedgerStore::fetch_run is not implemented");
test_panic!(test_fetch_runs, fetch_runs, "LedgerStore::fetch_runs is not implemented");
test_panic!(test_statement_lines_for_reconciliation_run, statement_lines_for_reconciliation_run, "run-1", "LedgerStore::statement_lines_for_reconciliation_run is not implemented");
test_panic!(test_reconciliation_run_count, reconciliation_run_count, "LedgerStore::reconciliation_run_count is not implemented");
test_panic!(test_reconciliation_run, reconciliation_run, "run-1", "LedgerStore::reconciliation_run is not implemented");
test_panic!(test_reconciliation_runs, reconciliation_runs, "LedgerStore::reconciliation_runs is not implemented");
test_panic!(test_month_close_count, month_close_count, "LedgerStore::month_close_count is not implemented");
test_panic!(test_month_close, month_close, "close-1", "LedgerStore::month_close is not implemented");
test_panic!(test_month_close_for_scope, month_close_for_scope, "2024-01", "Checking", "LedgerStore::month_close_for_scope is not implemented");
test_panic!(test_month_closes, month_closes, "LedgerStore::month_closes is not implemented");

test_error!(test_transactions_as_of_us, transactions_as_of_us, 0, 0, StoreError::LoadFailed, "LedgerStore::transactions_as_of_us is not implemented");

#[test]
fn test_write_transaction() {
    #[allow(unused_mut)]
    let mut store = DummyStore;
    let result = store.write_transaction(TransactionBuilder::new("test"));
    match result {
        Err(StoreError::PersistFailed { message }) => {
            assert_eq!(message, "LedgerStore::write_transaction is not implemented");
        }
        _ => panic!("Expected error not returned"),
    }
}

#[test]
fn test_write_transaction_with_valid_time() {
    #[allow(unused_mut)]
    let mut store = DummyStore;
    let result = store.write_transaction_with_valid_time(TransactionBuilder::new("test"), Some(0));
    match result {
        Err(StoreError::PersistFailed { message }) => {
            assert_eq!(message, "LedgerStore::write_transaction_with_valid_time is not implemented");
        }
        _ => panic!("Expected error not returned"),
    }
}

#[test]
fn test_write_correction() {
    #[allow(unused_mut)]
    let mut store = DummyStore;
    let tx_id = TransactionId::new("tx-123").unwrap();
    let result = store.write_correction(Correction::new(tx_id, "test").unwrap());
    match result {
        Err(StoreError::PersistFailed { message }) => {
            assert_eq!(message, "LedgerStore::write_correction is not implemented");
        }
        _ => panic!("Expected error not returned"),
    }
}

test_error!(test_write_budget_target, write_budget_target, "2024-01", "Expenses", 100, StoreError::PersistFailed, "LedgerStore::write_budget_target is not implemented");

#[test]
fn test_write_analytics_artifact_manifest() {
    #[allow(unused_mut)]
    let mut store = DummyStore;
    let result = store.write_analytics_artifact_manifest("kind", "uri", "hash", 1, 1, 1, 1, None);
    match result {
        Err(StoreError::PersistFailed { message }) => {
            assert_eq!(message, "LedgerStore::write_analytics_artifact_manifest is not implemented");
        }
        _ => panic!("Expected error not returned"),
    }
}

#[test]
fn test_write_analytics_artifact_manifest_us() {
    #[allow(unused_mut)]
    let mut store = DummyStore;
    let result = store.write_analytics_artifact_manifest_us("kind", "uri", "hash", 1, 1, 1, 1, None);
    match result {
        Err(StoreError::PersistFailed { message }) => {
            assert_eq!(message, "LedgerStore::write_analytics_artifact_manifest_us is not implemented");
        }
        _ => panic!("Expected error not returned"),
    }
}

#[test]
fn test_write_import_batch() {
    #[allow(unused_mut)]
    let mut store = DummyStore;
    let result = store.write_import_batch("kind", "uri", "batch", 0, false, false, &[]);
    match result {
        Err(StoreError::PersistFailed { message }) => {
            assert_eq!(message, "LedgerStore::write_import_batch is not implemented");
        }
        _ => panic!("Expected error not returned"),
    }
}

#[test]
fn test_write_fetch_run() {
    #[allow(unused_mut)]
    let mut store = DummyStore;
    let result = store.write_fetch_run("source", "inst", "acc", "2024-01", StoredFetchRunStatus::Downloaded, None, None, None, None, None);
    match result {
        Err(StoreError::PersistFailed { message }) => {
            assert_eq!(message, "LedgerStore::write_fetch_run is not implemented");
        }
        _ => panic!("Expected error not returned"),
    }
}

#[test]
fn test_write_reconciliation_run() {
    #[allow(unused_mut)]
    let mut store = DummyStore;
    let result = store.write_reconciliation_run("2024-01", "Checking", 0, 0, 0, 0, 0, true, 0, 0, 0, &[]);
    match result {
        Err(StoreError::PersistFailed { message }) => {
            assert_eq!(message, "LedgerStore::write_reconciliation_run is not implemented");
        }
        _ => panic!("Expected error not returned"),
    }
}

#[test]
fn test_write_reconciliation_run_and_month_close() {
    #[allow(unused_mut)]
    let mut store = DummyStore;
    let result = store.write_reconciliation_run_and_month_close("2024-01", "Checking", 0, 0, 0, 0, 0, true, 0, 0, 0, &[], None);
    match result {
        Err(StoreError::PersistFailed { message }) => {
            assert_eq!(message, "LedgerStore::write_reconciliation_run_and_month_close is not implemented");
        }
        _ => panic!("Expected error not returned"),
    }
}

#[test]
fn test_write_month_close() {
    #[allow(unused_mut)]
    let mut store = DummyStore;
    let result = store.write_month_close("2024-01", "Checking", "run-1", None);
    match result {
        Err(StoreError::PersistFailed { message }) => {
            assert_eq!(message, "LedgerStore::write_month_close is not implemented");
        }
        _ => panic!("Expected error not returned"),
    }
}
