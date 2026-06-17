#![allow(clippy::should_panic_without_expect)]
use logos_core::AccountId;
use logos_core::CategoryGroupId;
use logos_core::category_trends::CategoryTrendAnalyzer;
use logos_core::{Posting, TransactionBuilder};

#[test]
#[should_panic]
fn test_category_trends_panics_on_overflow() {
    let mut analyzer = CategoryTrendAnalyzer::new();
    let acc = AccountId::new("expenses:misc").unwrap();
    let group = CategoryGroupId::from_name("Group").unwrap();
    analyzer.map_account(acc.clone(), group);

    let tx = TransactionBuilder::new("Test")
        .posting(Posting::debit(acc.clone(), i64::MAX).unwrap())
        .posting(Posting::credit(AccountId::new("assets:checking").unwrap(), i64::MAX).unwrap())
        .build()
        .unwrap();

    let tx2 = TransactionBuilder::new("Test2")
        .posting(Posting::debit(acc, i64::MAX).unwrap())
        .posting(Posting::credit(AccountId::new("assets:checking").unwrap(), i64::MAX).unwrap())
        .build()
        .unwrap();

    let _ = analyzer.compute_spending_by_category(&[tx, tx2]);
}
