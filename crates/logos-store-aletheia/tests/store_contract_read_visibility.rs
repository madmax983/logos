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

#[test]
fn transactions_as_of_us_returns_non_empty_for_visible_transactions() {
    let path = temp_store_path("read-visibility-as-of-us");
    let mut store = AletheiaStore::open(&path).unwrap();
    let txn_id = store
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

    #[allow(clippy::cast_possible_truncation)]
    let valid_time_us = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_micros() as i64;
    #[allow(clippy::cast_possible_truncation)]
    let tx_time_us = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_micros() as i64;

    let transactions = store
        .transactions_as_of_us(valid_time_us + 10_000_000, tx_time_us + 10_000_000)
        .unwrap();
    assert_eq!(transactions.len(), 1);
    assert_eq!(transactions[0].id().as_str(), txn_id.as_str());
}

#[test]
fn has_visible_supersedes_edge_returns_false_when_not_superseded() {
    let path = temp_store_path("read-visibility-visible-supersedes");
    let mut store = AletheiaStore::open(&path).unwrap();

    let txn_id = store
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

    // Wait a bit to ensure transaction times are distinct
    std::thread::sleep(std::time::Duration::from_millis(5));
    let tx_time_between = aletheiadb::time::now();
    std::thread::sleep(std::time::Duration::from_millis(5));

    let correction = logos_core::Correction::new(txn_id.clone(), "fixed memo").unwrap();
    store.write_correction(correction).unwrap();

    // At `tx_time_between`, the transaction exists but the supersedes edge does NOT exist.
    // If `has_visible_supersedes_edge` is mutated to `Ok(true)`, it will erroneously filter it out.
    let transactions = store
        .transactions_as_of(aletheiadb::time::now(), tx_time_between)
        .unwrap();

    assert_eq!(transactions.len(), 1);
    assert_eq!(transactions[0].id().as_str(), txn_id.as_str());
}
