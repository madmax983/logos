use logos_core::AccountId;
use logos_core::domain::transaction::{Posting, TransactionBuilder};

#[test]
fn test_posting_account() {
    let posting = Posting::debit(AccountId::new("assets:checking").unwrap(), 1000).unwrap();
    assert_eq!(posting.account().as_str(), "assets:checking");
}

#[test]
fn test_transaction_description() {
    let txn = TransactionBuilder::new("Buy groceries")
        .posting(Posting::debit(AccountId::new("expenses:food").unwrap(), 5000).expect("debit"))
        .posting(Posting::credit(AccountId::new("assets:checking").unwrap(), 5000).unwrap())
        .build()
        .expect("Transaction should balance");

    assert_eq!(txn.description(), "Buy groceries");
}
