use aletheiadb::time;
use logos_core::{AccountId, Posting, TransactionBuilder};
use logos_store_aletheia::AletheiaStore;

fn temp_store_path(prefix: &str) -> std::path::PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!("logos-store-{prefix}-{nanos}.db"))
}

fn cleanup_store_path(path: &std::path::Path) {
    if path.exists() {
        if path.is_dir() {
            let _ = std::fs::remove_dir_all(path);
        } else {
            let _ = std::fs::remove_file(path);
        }
    }
}

#[test]
fn test_transactions_as_of_us_returns_items() {
    let path = temp_store_path("asof-us-test");
    let valid_time = time::from_secs(1_700_000_100);
    let mut store = AletheiaStore::open(&path).unwrap();

    let txn_id = store
        .write_transaction_with_valid_time(
            TransactionBuilder::new("test")
                .posting(Posting::debit(AccountId::new("assets:checking").unwrap(), 100).unwrap())
                .posting(Posting::credit(AccountId::new("income:salary").unwrap(), 100).unwrap()),
            Some(valid_time),
        )
        .unwrap();

    let _txn_id2 = store
        .write_transaction_with_valid_time(
            TransactionBuilder::new("test2")
                .posting(Posting::debit(AccountId::new("assets:checking").unwrap(), 200).unwrap())
                .posting(Posting::credit(AccountId::new("income:salary").unwrap(), 200).unwrap()),
            Some(valid_time),
        )
        .unwrap();

    let items = store
        .transactions_as_of_us(
            valid_time.wallclock() as i64,
            time::now().wallclock() as i64,
        )
        .unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0].id(), &txn_id);

    cleanup_store_path(&path);
}
