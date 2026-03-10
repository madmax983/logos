use logos_core::AccountId;
use logos_core::{Posting, TransactionBuilder};
use logos_store_aletheia::AletheiaStore;

fn temp_store_path(prefix: &str) -> std::path::PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!("logos-store-{prefix}-{nanos}.db"))
}

#[test]
fn db_node_not_visible_error_maps_to_none() {
    let path = temp_store_path("read-visibility-node");
    let mut store = AletheiaStore::open(&path).unwrap();
    let _txn_id = store
        .write_transaction(
            TransactionBuilder::new("paycheck")
                .posting(
                    Posting::debit(AccountId::new("assets:checking").unwrap(), 10_000).unwrap(),
                )
                .posting(
                    Posting::credit(AccountId::new("income:salary").unwrap(), 10_000).unwrap(),
                ),
        )
        .unwrap();

    let tx_time_before = aletheiadb::time::from_secs(1);

    // Testing `get_node_at_as_of` mapping to None requires us to try to read a transaction before it existed.
    let transactions = store
        .transactions_as_of(aletheiadb::time::now(), tx_time_before)
        .unwrap();
    assert!(transactions.is_empty());
}

#[test]
fn db_edge_not_visible_error_maps_to_none() {
    let path = temp_store_path("read-visibility-edge");
    let mut store = AletheiaStore::open(&path).unwrap();
    let _txn_id = store
        .write_transaction(
            TransactionBuilder::new("paycheck")
                .posting(
                    Posting::debit(AccountId::new("assets:checking").unwrap(), 10_000).unwrap(),
                )
                .posting(
                    Posting::credit(AccountId::new("income:salary").unwrap(), 10_000).unwrap(),
                ),
        )
        .unwrap();

    let tx_time_before = aletheiadb::time::from_secs(1);

    // In our `transactions_as_of` test above, since the transaction was not returned we didn't test loading its edges.
    // Let's create a transaction with an older effective time but newer tx_time, and try to query it with a valid time
    // after the effective time, but before the tx_time.
    let valid_time_before = aletheiadb::time::from_secs(100);
    let valid_time_now = aletheiadb::time::from_secs(200);

    store
        .write_transaction_with_valid_time(
            TransactionBuilder::new("backdated")
                .posting(
                    Posting::debit(AccountId::new("assets:checking").unwrap(), 10_000).unwrap(),
                )
                .posting(
                    Posting::credit(AccountId::new("income:salary").unwrap(), 10_000).unwrap(),
                ),
            Some(valid_time_before),
        )
        .unwrap();

    let _tx_time_between = aletheiadb::time::now();

    // Now, query with a valid time where the transaction should exist (after valid_time_before)
    // but with a tx_time where the transaction should not exist (before the backdated transaction was written).
    // This will exercise the edge path.
    let transactions = store
        .transactions_as_of(valid_time_now, tx_time_before)
        .unwrap();

    // the backdated transaction shouldn't be there.
    assert_eq!(transactions.len(), 0);
}
