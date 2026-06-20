#![allow(clippy::should_panic_without_expect)]
use logos_core::category_trends::CategoryTrendAnalyzer;
use logos_core::{AccountId, CategoryGroupId, Posting, TransactionBuilder};
use proptest::prelude::*;

proptest! {
    #[test]
    #[should_panic(expected = "attempt to add with overflow")]
    fn havoc_category_trends_panics_on_overflow(
        amount1 in (i64::MAX / 2 + 1)..=i64::MAX,
        amount2 in (i64::MAX / 2 + 1)..=i64::MAX,
    ) {
        let mut analyzer = CategoryTrendAnalyzer::new();
        let rent_acc = AccountId::new("expenses:rent").unwrap();
        let checking_acc = AccountId::new("assets:checking").unwrap();
        let housing_group = CategoryGroupId::from_name("Housing").unwrap();

        analyzer.map_account(rent_acc.clone(), housing_group);

        let tx1 = TransactionBuilder::new("Rent 1")
            .posting(Posting::debit(rent_acc.clone(), amount1).unwrap())
            .posting(Posting::credit(checking_acc.clone(), amount1).unwrap())
            .build()
            .unwrap();

        let tx2 = TransactionBuilder::new("Rent 2")
            .posting(Posting::debit(rent_acc, amount2).unwrap())
            .posting(Posting::credit(checking_acc, amount2).unwrap())
            .build()
            .unwrap();

        let _ = analyzer.compute_spending_by_category(&[tx1, tx2]);
    }
}
