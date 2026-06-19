#![cfg(feature = "nova")]
#![allow(clippy::should_panic_without_expect)]
use logos_core::AccountId;
use logos_core::CategoryGroupId;
use logos_core::category_trends::CategoryTrendAnalyzer;
use logos_core::{Posting, TransactionBuilder};
use proptest::prelude::*;

proptest! {
    #[test]
    #[should_panic]
    fn category_trends_panics_on_overflow(
        amount in (i64::MAX / 2 + 1)..=i64::MAX,
    ) {
        let mut analyzer = CategoryTrendAnalyzer::new();
        let acc = AccountId::new("expenses:food").unwrap();
        let group = CategoryGroupId::from_name("Food").unwrap();
        analyzer.map_account(acc.clone(), group);

        let tx1 = TransactionBuilder::new("Groceries")
            .posting(Posting::debit(acc.clone(), amount).unwrap())
            .posting(Posting::credit(AccountId::new("assets:checking").unwrap(), amount).unwrap())
            .build()
            .unwrap();

        let tx2 = TransactionBuilder::new("Groceries")
            .posting(Posting::debit(acc, amount).unwrap())
            .posting(Posting::credit(AccountId::new("assets:checking").unwrap(), amount).unwrap())
            .build()
            .unwrap();

        let _ = analyzer.compute_spending_by_category(&[tx1, tx2]);
    }
}
