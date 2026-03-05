use logos_core::domain::budget::rollover_end_balance;
use logos_core::domain::transaction::{Posting, TransactionBuilder};
use proptest::prelude::*;

proptest! {
    #[test]
    fn havoc_posting_credit_does_not_panic(amount in any::<i64>()) {
        let _ = Posting::credit("test", amount);
    }

    #[test]
    fn havoc_transaction_builder_sum_does_not_panic(
        amounts in prop::collection::vec(any::<i64>(), 1..100)
    ) {
        let mut builder = TransactionBuilder::new("Test");
        for amt in amounts {
            builder = builder.posting(Posting::debit("test", amt));
        }
        let _ = builder.build();
    }

    #[test]
    fn havoc_rollover_end_balance_does_not_panic(
        start in any::<i64>(),
        assigned in any::<i64>(),
        spent in any::<i64>()
    ) {
        let _ = rollover_end_balance(start, assigned, spent);
    }
}

#[test]
fn havoc_posting_credit_min_does_not_panic() {
    let _ = logos_core::domain::transaction::Posting::credit("test", i64::MIN);
}

use logos_core::domain::rsu::AllocationPolicy;
use logos_core::planning::rsu_distributor::{RsuAutoDistributor, RsuDistributorConfig};
use logos_core::domain::account::AccountId;

proptest! {
    #[test]
    fn havoc_rsu_distribute_does_not_panic(
        gross in any::<i64>(),
        tax in 0..=100u8,
        smooth in 0..=100u8,
        goals in 0..=100u8,
        disc in 0..=100u8
    ) {
        if let Ok(policy) = AllocationPolicy::new(tax, smooth, goals, disc) {
            let config = RsuDistributorConfig {
                rsu_asset: AccountId::new("assets:rsu"),
                tax_reserve: AccountId::new("assets:tax"),
                smoothing_buffer: AccountId::new("assets:buffer"),
                goals: AccountId::new("assets:goals"),
                discretionary: AccountId::new("assets:checking"),
            };
            let distributor = RsuAutoDistributor::new(config);
            let _ = distributor.distribute_rsu_vest("Vest", gross, &policy);
        }
    }
}
