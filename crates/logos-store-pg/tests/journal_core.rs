use std::time::{Duration, SystemTime, UNIX_EPOCH};

use logos_core::{AccountId, Correction, Posting, TransactionBuilder, TransactionId};
use logos_store::LedgerStore;
use logos_store::StoreError;
use logos_store_pg::PostgresStore;
use testcontainers_modules::{postgres, testcontainers::runners::SyncRunner};

fn balanced_transaction(description: &str, amount_cents: i64) -> TransactionBuilder {
    TransactionBuilder::new(description)
        .posting(Posting::debit(AccountId::new("assets:checking").unwrap(), amount_cents).unwrap())
        .posting(Posting::credit(AccountId::new("income:salary").unwrap(), amount_cents).unwrap())
}

fn now_us() -> i64 {
    i64::try_from(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_micros(),
    )
    .expect("microseconds must fit into i64")
}

fn connect_store(database_url: &str) -> PostgresStore {
    let mut store = PostgresStore::connect(database_url).expect("connect postgres store");
    store.run_migrations().expect("run migrations");
    store
}

#[test]
#[ignore = "requires Docker (testcontainers)"]
fn write_transaction_persists_across_reconnect_and_ids_are_monotonic() {
    let container = postgres::Postgres::default()
        .start()
        .expect("start postgres container");
    let host = container.get_host().expect("container host");
    let port = container.get_host_port_ipv4(5432).expect("postgres port");
    let database_url = format!("postgres://postgres:postgres@{host}:{port}/postgres");

    let mut store = connect_store(&database_url);
    let first_id = store
        .write_transaction(balanced_transaction("first", 10_000))
        .expect("write first transaction");
    assert_eq!(first_id.as_str(), "txn-1");
    assert_eq!(store.transaction_count(), 1);
    drop(store);

    let mut reopened = PostgresStore::connect(&database_url).expect("reconnect postgres store");
    assert_eq!(reopened.transaction_count(), 1);
    assert!(reopened.has_transaction(&first_id));

    let second_id = reopened
        .write_transaction(balanced_transaction("second", 20_000))
        .expect("write second transaction");
    assert_eq!(second_id.as_str(), "txn-2");
    assert_eq!(reopened.transaction_count(), 2);
    assert!(reopened.has_transaction(&first_id));
    assert!(reopened.has_transaction(&second_id));
}

#[test]
#[ignore = "requires Docker (testcontainers)"]
fn write_transaction_rejects_unbalanced_input_without_persistence() {
    let container = postgres::Postgres::default()
        .start()
        .expect("start postgres container");
    let host = container.get_host().expect("container host");
    let port = container.get_host_port_ipv4(5432).expect("postgres port");
    let database_url = format!("postgres://postgres:postgres@{host}:{port}/postgres");

    let mut store = connect_store(&database_url);
    let err = store
        .write_transaction(
            TransactionBuilder::new("bad")
                .posting(
                    Posting::debit(AccountId::new("assets:checking").unwrap(), 10_000).unwrap(),
                )
                .posting(Posting::credit(AccountId::new("income:salary").unwrap(), 9_000).unwrap()),
        )
        .expect_err("unbalanced transactions must fail");

    assert!(matches!(err, StoreError::Domain(_)));
    assert_eq!(store.transaction_count(), 0);
}

#[test]
#[ignore = "requires Docker (testcontainers)"]
fn transactions_as_of_us_respects_backdated_valid_time_and_write_time() {
    let container = postgres::Postgres::default()
        .start()
        .expect("start postgres container");
    let host = container.get_host().expect("container host");
    let port = container.get_host_port_ipv4(5432).expect("postgres port");
    let database_url = format!("postgres://postgres:postgres@{host}:{port}/postgres");

    let mut store = connect_store(&database_url);
    let valid_at = 1_700_000_000_000_000_i64;
    let tx_before_write = now_us();
    std::thread::sleep(Duration::from_millis(5));

    let txn_id = store
        .write_transaction_with_valid_time(balanced_transaction("backdated", 7_500), Some(valid_at))
        .expect("write backdated transaction");

    let before_valid = store
        .transactions_as_of_us(valid_at - 1, i64::MAX)
        .expect("query before valid time");
    assert!(before_valid.is_empty());

    let before_write = store
        .transactions_as_of_us(valid_at, tx_before_write)
        .expect("query before write tx time");
    assert!(before_write.is_empty());

    let visible = store
        .transactions_as_of_us(valid_at, now_us())
        .expect("query at visible point");
    assert_eq!(visible.len(), 1);
    assert_eq!(visible[0].id(), &txn_id);
}

#[test]
#[ignore = "requires Docker (testcontainers)"]
fn write_correction_requires_known_transaction() {
    let container = postgres::Postgres::default()
        .start()
        .expect("start postgres container");
    let host = container.get_host().expect("container host");
    let port = container.get_host_port_ipv4(5432).expect("postgres port");
    let database_url = format!("postgres://postgres:postgres@{host}:{port}/postgres");

    let mut store = connect_store(&database_url);
    let correction = Correction::new(TransactionId::new("txn-999").unwrap(), "reason").unwrap();
    let err = store
        .write_correction(correction)
        .expect_err("unknown transaction correction must fail");

    assert!(matches!(err, StoreError::UnknownTransaction { .. }));
    assert_eq!(store.correction_count(), 0);
}

#[test]
#[ignore = "requires Docker (testcontainers)"]
fn transactions_as_of_us_hides_superseded_after_correction_tx_time() {
    let container = postgres::Postgres::default()
        .start()
        .expect("start postgres container");
    let host = container.get_host().expect("container host");
    let port = container.get_host_port_ipv4(5432).expect("postgres port");
    let database_url = format!("postgres://postgres:postgres@{host}:{port}/postgres");

    let mut store = connect_store(&database_url);
    let txn_id = store
        .write_transaction(balanced_transaction("paycheck", 10_000))
        .expect("write transaction");
    assert_eq!(store.transactions().len(), 1);

    std::thread::sleep(Duration::from_millis(5));
    let tx_before_correction = now_us();
    std::thread::sleep(Duration::from_millis(5));

    store
        .write_correction(Correction::new(txn_id.clone(), "fix memo").unwrap())
        .expect("write correction");

    let latest = store.latest_correction().expect("latest correction");
    assert_eq!(latest.supersedes_id(), &txn_id);
    assert_eq!(store.correction_count(), 1);
    assert_eq!(store.transactions().len(), 1);

    let before_correction = store
        .transactions_as_of_us(i64::MAX, tx_before_correction)
        .expect("query before correction");
    assert!(
        before_correction
            .iter()
            .any(|stored| stored.id() == &txn_id)
    );

    let after_correction = store
        .transactions_as_of_us(i64::MAX, now_us())
        .expect("query after correction");
    assert!(!after_correction.iter().any(|stored| stored.id() == &txn_id));
}
