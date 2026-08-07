use logos_core::{Correction, TransactionBuilder, TransactionId};
use logos_store::{LedgerStore, StoreError};

struct DummyStore;
impl LedgerStore for DummyStore {}

#[test]
#[should_panic(expected = "LedgerStore::transaction_count is not implemented")]
fn should_panic_when_transaction_count_called() {
    let store = DummyStore;
    store.transaction_count();
}

#[test]
#[should_panic(expected = "LedgerStore::correction_count is not implemented")]
fn should_panic_when_correction_count_called() {
    let store = DummyStore;
    store.correction_count();
}

#[test]
#[should_panic(expected = "LedgerStore::has_transaction is not implemented")]
fn should_panic_when_has_transaction_called() {
    let store = DummyStore;
    let id = TransactionId::new("txn-1").expect("id");
    store.has_transaction(&id);
}

#[test]
#[should_panic(expected = "LedgerStore::latest_correction is not implemented")]
fn should_panic_when_latest_correction_called() {
    let store = DummyStore;
    store.latest_correction();
}

#[test]
#[should_panic(expected = "LedgerStore::transactions is not implemented")]
fn should_panic_when_transactions_called() {
    let store = DummyStore;
    store.transactions();
}

#[test]
#[should_panic(expected = "LedgerStore::budget_target is not implemented")]
fn should_panic_when_budget_target_called() {
    let store = DummyStore;
    store.budget_target("2023-01", "expenses:food");
}

#[test]
#[should_panic(expected = "LedgerStore::budget_targets is not implemented")]
fn should_panic_when_budget_targets_called() {
    let store = DummyStore;
    store.budget_targets();
}

#[test]
#[should_panic(expected = "LedgerStore::analytics_artifact is not implemented")]
fn should_panic_when_analytics_artifact_called() {
    let store = DummyStore;
    store.analytics_artifact("artifact-1");
}

#[test]
#[should_panic(expected = "LedgerStore::analytics_artifacts is not implemented")]
fn should_panic_when_analytics_artifacts_called() {
    let store = DummyStore;
    store.analytics_artifacts();
}

#[test]
#[should_panic(expected = "LedgerStore::import_record_count is not implemented")]
fn should_panic_when_import_record_count_called() {
    let store = DummyStore;
    store.import_record_count();
}

#[test]
#[should_panic(expected = "LedgerStore::has_import_record_content_hash is not implemented")]
fn should_panic_when_has_import_record_content_hash_called() {
    let store = DummyStore;
    store.has_import_record_content_hash("hash");
}

#[test]
#[should_panic(expected = "LedgerStore::import_records is not implemented")]
fn should_panic_when_import_records_called() {
    let store = DummyStore;
    store.import_records();
}

#[test]
#[should_panic(expected = "LedgerStore::import_batches is not implemented")]
fn should_panic_when_import_batches_called() {
    let store = DummyStore;
    store.import_batches();
}

#[test]
#[should_panic(expected = "LedgerStore::statement_line_count is not implemented")]
fn should_panic_when_statement_line_count_called() {
    let store = DummyStore;
    store.statement_line_count();
}

#[test]
#[should_panic(expected = "LedgerStore::statement_lines is not implemented")]
fn should_panic_when_statement_lines_called() {
    let store = DummyStore;
    store.statement_lines();
}

#[test]
#[should_panic(expected = "LedgerStore::fetch_run_count is not implemented")]
fn should_panic_when_fetch_run_count_called() {
    let store = DummyStore;
    store.fetch_run_count();
}

#[test]
#[should_panic(expected = "LedgerStore::fetch_run is not implemented")]
fn should_panic_when_fetch_run_called() {
    let store = DummyStore;
    store.fetch_run("run-1");
}

#[test]
#[should_panic(expected = "LedgerStore::fetch_runs is not implemented")]
fn should_panic_when_fetch_runs_called() {
    let store = DummyStore;
    store.fetch_runs();
}

#[test]
#[should_panic(expected = "LedgerStore::statement_lines_for_reconciliation_run is not implemented")]
fn should_panic_when_statement_lines_for_reconciliation_run_called() {
    let store = DummyStore;
    store.statement_lines_for_reconciliation_run("run-1");
}

#[test]
#[should_panic(expected = "LedgerStore::reconciliation_run_count is not implemented")]
fn should_panic_when_reconciliation_run_count_called() {
    let store = DummyStore;
    store.reconciliation_run_count();
}

#[test]
#[should_panic(expected = "LedgerStore::reconciliation_run is not implemented")]
fn should_panic_when_reconciliation_run_called() {
    let store = DummyStore;
    store.reconciliation_run("run-1");
}

#[test]
#[should_panic(expected = "LedgerStore::reconciliation_runs is not implemented")]
fn should_panic_when_reconciliation_runs_called() {
    let store = DummyStore;
    store.reconciliation_runs();
}

#[test]
#[should_panic(expected = "LedgerStore::month_close_count is not implemented")]
fn should_panic_when_month_close_count_called() {
    let store = DummyStore;
    store.month_close_count();
}

#[test]
#[should_panic(expected = "LedgerStore::month_close is not implemented")]
fn should_panic_when_month_close_called() {
    let store = DummyStore;
    store.month_close("close-1");
}

#[test]
#[should_panic(expected = "LedgerStore::month_close_for_scope is not implemented")]
fn should_panic_when_month_close_for_scope_called() {
    let store = DummyStore;
    store.month_close_for_scope("2023-01", "checking");
}

#[test]
#[should_panic(expected = "LedgerStore::month_closes is not implemented")]
fn should_panic_when_month_closes_called() {
    let store = DummyStore;
    store.month_closes();
}

#[test]
fn should_return_error_when_transactions_as_of_us_called() {
    let store = DummyStore;
    let res = store.transactions_as_of_us(0, 0);
    assert_eq!(
        res.unwrap_err(),
        StoreError::LoadFailed {
            message: "LedgerStore::transactions_as_of_us is not implemented".to_string(),
        }
    );
}

#[test]
fn should_return_error_when_write_transaction_called() {
    let mut store = DummyStore;
    let builder = TransactionBuilder::new("desc");
    let res = store.write_transaction(builder);
    assert_eq!(
        res.unwrap_err(),
        StoreError::PersistFailed {
            message: "LedgerStore::write_transaction is not implemented".to_string(),
        }
    );
}

#[test]
fn should_return_error_when_write_transaction_with_valid_time_called() {
    let mut store = DummyStore;
    let builder = TransactionBuilder::new("desc");
    let res = store.write_transaction_with_valid_time(builder, None);
    assert_eq!(
        res.unwrap_err(),
        StoreError::PersistFailed {
            message: "LedgerStore::write_transaction_with_valid_time is not implemented".to_string(),
        }
    );
}

#[test]
fn should_return_error_when_write_correction_called() {
    let mut store = DummyStore;
    let id = TransactionId::new("txn-1").expect("id");
    let correction = Correction::new(id, "memo").expect("correction");
    let res = store.write_correction(correction);
    assert_eq!(
        res.unwrap_err(),
        StoreError::PersistFailed {
            message: "LedgerStore::write_correction is not implemented".to_string(),
        }
    );
}

#[test]
fn should_return_error_when_write_budget_target_called() {
    let mut store = DummyStore;
    let res = store.write_budget_target("2023-01", "expenses", 100);
    assert_eq!(
        res.unwrap_err(),
        StoreError::PersistFailed {
            message: "LedgerStore::write_budget_target is not implemented".to_string(),
        }
    );
}
