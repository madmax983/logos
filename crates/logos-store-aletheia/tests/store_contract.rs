use logos_core::{Correction, Posting, TransactionBuilder};
use logos_store_aletheia::AletheiaStore;

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
    store.write_correction(correction).expect("append correction");

    let latest = store.latest_correction().expect("correction exists");
    assert_eq!(latest.supersedes_id(), &id);
}
