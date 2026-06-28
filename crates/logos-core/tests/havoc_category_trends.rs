#![allow(clippy::should_panic_without_expect)]
#![cfg(feature = "nova")]
use logos_core::category_trends::CategoryTrendAnalyzer;
use logos_core::domain::account::AccountId;
use logos_core::domain::category::CategoryGroupId;
use logos_core::domain::transaction::{Posting, TransactionBuilder};
use proptest::prelude::*;

proptest! {
    #[test]
    #[should_panic(expected = "attempt to add with overflow")]
    fn category_trends_panics_on_overflow(
        amount in (i64::MAX / 2 + 1)..=i64::MAX,
    ) {
        let mut analyzer = CategoryTrendAnalyzer::new();
        let acc = AccountId::new("expenses:misc").unwrap();
        let group = CategoryGroupId::from_name("Group").unwrap();
        analyzer.map_account(acc.clone(), group);

        let tx1 = TransactionBuilder::new("Tx 1")
            .posting(Posting::debit(acc.clone(), amount).unwrap())
            .posting(Posting::credit(AccountId::new("assets:checking").unwrap(), amount).unwrap())
            .build()
            .unwrap();

        let tx2 = TransactionBuilder::new("Tx 2")
            .posting(Posting::debit(acc, amount).unwrap())
            .posting(Posting::credit(AccountId::new("assets:checking").unwrap(), amount).unwrap())
            .build()
            .unwrap();

        let _ = analyzer.compute_spending_by_category(&[tx1, tx2]);
    }
}
