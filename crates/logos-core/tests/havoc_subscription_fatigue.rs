#![cfg(feature = "nova")]
#![allow(clippy::should_panic_without_expect)]

use logos_core::subscription_fatigue::SubscriptionFatigueAnalyzer;
use logos_core::AccountId;
use logos_core::{Posting, TransactionBuilder};
use proptest::prelude::*;

proptest! {
    #[test]
    #[should_panic]
    fn subscription_fatigue_panics_on_overflow(
        amount in (i64::MAX / 2 + 1)..=i64::MAX,
    ) {
        let analyzer = SubscriptionFatigueAnalyzer::new(1, 7.0, 10);
        let mut transactions = Vec::new();

        let tx1 = TransactionBuilder::new("Massive Sub 1")
            .posting(Posting::credit(AccountId::new("assets:checking").unwrap(), amount).unwrap())
            .posting(Posting::debit(AccountId::new("expenses:sub").unwrap(), amount).unwrap())
            .build()
            .unwrap();
        transactions.push(tx1);

        let tx2 = TransactionBuilder::new("Massive Sub 2")
            .posting(Posting::credit(AccountId::new("assets:checking").unwrap(), amount).unwrap())
            .posting(Posting::debit(AccountId::new("expenses:sub").unwrap(), amount).unwrap())
            .build()
            .unwrap();
        transactions.push(tx2);

        let _ = analyzer.analyze(&transactions);
    }
}
