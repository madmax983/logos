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

    #[test]
    fn havoc_fire_simulator_safe_net_worth_does_not_panic(
        expenses in any::<i64>(),
        assets in any::<i64>(),
        liabilities in any::<i64>(),
        vests in prop::collection::vec(
            (any::<i64>(), any::<u32>(), any::<u16>()),
            0..1000
        )
    ) {
        let mut sim = logos_core::planning::fire::FireSimulator::new(expenses);
        sim.add_assets_liabilities(assets, liabilities);
        for (price, units, days) in vests {
            sim.add_upcoming_vest(logos_core::planning::fire::UpcomingVest {
                avg_close_price_cents: price,
                units,
                days_to_vest: days,
            });
        }
        let _ = sim.safe_net_worth_cents();
    }
}

#[test]
fn havoc_posting_credit_min_does_not_panic() {
    let _ = logos_core::domain::transaction::Posting::credit("test", i64::MIN);
}

#[test]
#[should_panic]
fn havoc_fire_simulator_safe_net_worth_overflow_panics() {
    let mut sim = logos_core::planning::fire::FireSimulator::new(5000);
    let vest = logos_core::planning::fire::UpcomingVest {
        avg_close_price_cents: i64::MAX,
        units: 2,
        days_to_vest: 10,
    };
    for _ in 0..200 {
        sim.add_upcoming_vest(vest);
    }
    let _ = sim.safe_net_worth_cents();
}
