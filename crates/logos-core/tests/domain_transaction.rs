use logos_core::domain::transaction::{Posting, TransactionBuilder};

#[test]
fn test_posting_account() {
    let posting = Posting::debit("assets:checking", 1000);
    assert_eq!(posting.account(), "assets:checking");
}

#[test]
fn test_transaction_description() {
    let txn = TransactionBuilder::new("Buy groceries")
        .posting(Posting::debit("expenses:food", 5000))
        .posting(Posting::credit("assets:checking", 5000).unwrap())
        .build()
        .expect("Transaction should balance");

    assert_eq!(txn.description(), "Buy groceries");
}
