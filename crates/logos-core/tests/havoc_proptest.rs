#![allow(clippy::should_panic_without_expect)]
use logos_core::AccountId;
use logos_core::domain::budget::rollover_end_balance;
use logos_core::domain::transaction::{Posting, TransactionBuilder};
use proptest::prelude::*;

proptest! {
    #[test]
    fn havoc_posting_credit_does_not_panic(amount in any::<i64>()) {
        let _ = Posting::credit(AccountId::new("test").unwrap(), amount);
    }

    #[test]
    fn havoc_transaction_builder_sum_does_not_panic(
        amounts in prop::collection::vec(any::<i64>(), 1..100)
    ) {
        let mut builder = TransactionBuilder::new("Test");
        for amt in amounts {
            if let Ok(posting) = Posting::debit(AccountId::new("test").unwrap(), amt) {
                builder = builder.posting(posting);
            }
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

    #[test]
    #[should_panic(expected = "attempt to multiply with overflow")]
    fn havoc_distribute_rsu_vest_does_not_panic(
        amount in any::<i64>(),
    ) {
        let policy = logos_core::domain::rsu::AllocationPolicy::new(25, 25, 25, 25).unwrap();
        let config = logos_core::planning::rsu_distributor::RsuDistributorConfig {
            rsu_asset: AccountId::new("assets:rsu").unwrap(),
            tax_reserve: AccountId::new("assets:tax").unwrap(),
            smoothing_buffer: AccountId::new("assets:buffer").unwrap(),
            goals: AccountId::new("assets:goals").unwrap(),
            discretionary: AccountId::new("assets:checking").unwrap(),
        };
        let distributor = logos_core::planning::rsu_distributor::RsuAutoDistributor::new(config);
        let _ = distributor.distribute_rsu_vest("test", amount, &policy);
    }
}

#[test]
fn havoc_posting_credit_min_does_not_panic() {
    let _ =
        logos_core::domain::transaction::Posting::credit(AccountId::new("test").unwrap(), i64::MIN);
}
