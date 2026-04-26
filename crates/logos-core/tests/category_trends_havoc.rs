#![cfg(feature = "nova")]
#![allow(clippy::should_panic_without_expect)]
use logos_core::domain::account::AccountId;
use logos_core::domain::category::CategoryGroupId;
use logos_core::domain::transaction::{Posting, TransactionBuilder};
use logos_core::experimental::category_trends::CategoryTrendAnalyzer;
use proptest::prelude::*;

proptest! {
    #[test]
    #[should_panic(expected = "attempt to add with overflow")]
    fn category_trends_panics_on_overflow(
        amount in (i64::MAX / 2 + 1)..=i64::MAX,
    ) {
        let mut analyzer = CategoryTrendAnalyzer::new();
        let rent_acc = AccountId::new("expenses:rent").unwrap();
        let housing_group = CategoryGroupId::from_name("Housing").unwrap();
        let checking_acc = AccountId::new("assets:checking").unwrap();

        analyzer.map_account(rent_acc.clone(), housing_group.clone());

        let tx1 = TransactionBuilder::new("Rent")
            .posting(Posting::debit(rent_acc.clone(), amount).unwrap())
            .posting(Posting::credit(checking_acc.clone(), amount).unwrap())
            .build()
            .unwrap();

        let tx2 = TransactionBuilder::new("Rent2")
            .posting(Posting::debit(rent_acc.clone(), amount).unwrap())
            .posting(Posting::credit(checking_acc.clone(), amount).unwrap())
            .build()
            .unwrap();

        let _ = analyzer.compute_spending_by_category(&[tx1, tx2]);
    }
}
