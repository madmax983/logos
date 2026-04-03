use logos_core::{AccountId, Posting, TransactionBuilder, TransactionId};
use logos_store::error::StoreError;
use logos_store::model::{
    StoredFetchArtifactFormat, StoredFetchRun, StoredFetchRunStatus, StoredTransaction,
};
use logos_store::traits::LedgerStore;

#[allow(dead_code)]
fn _touch_runtime_contract<T: LedgerStore>(store: &mut T) {
    let _ = store.transaction_count();
    let _ = store.transactions_as_of_us(0, 0);
    let _ = store.write_fetch_run(
        "source-1",
        "bank-1",
        "assets:checking",
        "2026-03",
        StoredFetchRunStatus::Downloaded,
        None,
        None,
        None,
        None,
        None,
    );
    let _ = store.write_import_batch(
        "csv-statement",
        "inline:csv",
        "batch-1",
        0,
        false,
        false,
        &[],
    );
    let _ = store.write_reconciliation_run(
        "2026-03",
        "assets:checking",
        0,
        0,
        0,
        0,
        0,
        true,
        0,
        0,
        0,
        &[],
    );
    let _ = store.write_reconciliation_run_and_month_close(
        "2026-03",
        "assets:checking",
        0,
        0,
        0,
        0,
        0,
        true,
        0,
        0,
        0,
        &[],
        None,
    );
    let _ = store.write_month_close("2026-03", "assets:checking", "run-1", None);
}

#[test]
#[allow(clippy::items_after_statements)]
fn smoke_models_and_trait_object_compile() {
    let txn_id = TransactionId::new("txn-1").expect("valid transaction id");
    let transaction = TransactionBuilder::new("paycheck")
        .posting(
            Posting::debit(AccountId::new("assets:checking").expect("account"), 10_000)
                .expect("debit"),
        )
        .posting(
            Posting::credit(AccountId::new("income:salary").expect("account"), 10_000)
                .expect("credit"),
        )
        .build()
        .expect("balanced transaction");

    let stored_txn = StoredTransaction::with_effective_at(txn_id.clone(), transaction, 42);
    assert_eq!(stored_txn.id(), &txn_id);
    assert_eq!(stored_txn.effective_at(), 42);
    assert_eq!(stored_txn.transaction().postings().len(), 2);

    let fetch_run = StoredFetchRun::new(
        "fetch-1",
        "source-1",
        "bank-1",
        "assets:checking",
        "2026-03",
        StoredFetchRunStatus::Downloaded,
        Some("/tmp/fetch.csv"),
        Some(StoredFetchArtifactFormat::Csv),
        Some(1_000),
        Some(2_000),
        None,
        99,
    );
    assert_eq!(fetch_run.run_id(), "fetch-1");
    assert_eq!(fetch_run.status(), StoredFetchRunStatus::Downloaded);
    assert!(fetch_run.output_format().is_some());

    let err = StoreError::LoadFailed {
        message: "boom".to_owned(),
    };
    assert_eq!(err.to_string(), "failed to load store: boom");

    #[allow(clippy::items_after_statements)]
    fn accepts_trait_object(_store: &dyn LedgerStore) {}

    #[allow(clippy::items_after_statements)]
    struct DummyStore;
    #[allow(clippy::items_after_statements)]
    impl LedgerStore for DummyStore {}

    let store = DummyStore;
    accepts_trait_object(&store);
}
