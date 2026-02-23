use logos_core::{Posting, TransactionBuilder};

#[test]
fn balanced_transaction_is_accepted() {
    let txn = TransactionBuilder::new("paycheck")
        .posting(Posting::debit("assets:checking", 10_000))
        .posting(Posting::credit("income:salary", 10_000))
        .build()
        .expect("balanced");

    assert_eq!(txn.postings().len(), 2);
}

#[test]
fn unbalanced_transaction_is_rejected() {
    let err = TransactionBuilder::new("bad")
        .posting(Posting::debit("assets:checking", 10_000))
        .posting(Posting::credit("income:salary", 9_000))
        .build()
        .expect_err("must fail");

    assert!(err.to_string().contains("balanced"));
}
