#![cfg(feature = "nova")]
#![allow(clippy::should_panic_without_expect)]

use logos_core::subscription_fatigue::SubscriptionFatigueAnalyzer;
use logos_core::{AccountId, Posting, TransactionBuilder};
use proptest::prelude::*;

proptest! {
    #[test]
    #[should_panic(expected = "attempt to add with overflow")]
    fn subscription_fatigue_panics_on_overflow(
        cost1 in (i64::MAX / 2 + 1)..=i64::MAX,
        cost2 in (i64::MAX / 2 + 1)..=i64::MAX,
    ) {
        let analyzer = SubscriptionFatigueAnalyzer::new(1, 0.0, 1);

        let tx1 = TransactionBuilder::new("Netflix")
            .posting(Posting::debit(AccountId::new("expenses:subscriptions").unwrap(), cost1).unwrap())
            .posting(Posting::credit(AccountId::new("assets:checking").unwrap(), cost1).unwrap())
            .build()
            .unwrap();

        let tx2 = TransactionBuilder::new("Spotify")
            .posting(Posting::debit(AccountId::new("expenses:subscriptions").unwrap(), cost2).unwrap())
            .posting(Posting::credit(AccountId::new("assets:checking").unwrap(), cost2).unwrap())
            .build()
            .unwrap();

        let _ = analyzer.analyze(&[tx1, tx2]);
    }
}
