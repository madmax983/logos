use logos_core::AccountId;
use logos_core::{Posting, TransactionBuilder};

#[test]
fn balanced_transaction_is_accepted() {
    let txn = TransactionBuilder::new("paycheck")
        .posting(Posting::debit(
            AccountId::new("assets:checking").unwrap(),
            10_000,
        ))
        .posting(Posting::credit(AccountId::new("income:salary").unwrap(), 10_000).expect("credit"))
        .build()
        .expect("balanced");

    assert_eq!(txn.postings().len(), 2);
}

#[test]
fn unbalanced_transaction_is_rejected() {
    let err = TransactionBuilder::new("bad")
        .posting(Posting::debit(
            AccountId::new("assets:checking").unwrap(),
            10_000,
        ))
        .posting(Posting::credit(AccountId::new("income:salary").unwrap(), 9_000).expect("credit"))
        .build()
        .expect_err("must fail");

    assert!(err.to_string().contains("balanced"));
}

#[test]
fn empty_transaction_description_is_rejected() {
    let err = TransactionBuilder::new("   ")
        .posting(Posting::debit(
            AccountId::new("assets:checking").unwrap(),
            10_000,
        ))
        .posting(Posting::credit(AccountId::new("income:salary").unwrap(), 10_000).expect("credit"))
        .build()
        .expect_err("must fail");

    assert_eq!(err.to_string(), "transaction description cannot be empty");
}
