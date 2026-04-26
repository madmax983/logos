#![allow(clippy::should_panic_without_expect)]
#![cfg(feature = "nova")]
use logos_core::domain::account::AccountId;
use logos_core::domain::transaction::{Posting, TransactionBuilder};
use logos_core::experimental::subscription_fatigue::SubscriptionFatigueAnalyzer;
use proptest::prelude::*;

proptest! {
    #[test]
    #[should_panic(expected = "attempt to add with overflow")]
    fn subscription_fatigue_panics_on_overflow(
        amount in (i64::MAX / 2 + 1)..=i64::MAX,
    ) {
        let analyzer = SubscriptionFatigueAnalyzer::new(1, 7.0, 10);
        let mut transactions = Vec::new();

        let tx1 = TransactionBuilder::new("Sub1")
            .posting(Posting::credit(AccountId::new("assets:checking").unwrap(), amount).unwrap())
            .posting(Posting::debit(AccountId::new("expenses:entertainment").unwrap(), amount).unwrap())
            .build()
            .unwrap();
        transactions.push(tx1);

        let tx2 = TransactionBuilder::new("Sub2")
            .posting(Posting::credit(AccountId::new("assets:checking").unwrap(), amount).unwrap())
            .posting(Posting::debit(AccountId::new("expenses:entertainment").unwrap(), amount).unwrap())
            .build()
            .unwrap();
        transactions.push(tx2);

        let _report = analyzer.analyze(&transactions);
    }
}
