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
    fn havoc_net_worth_projector_does_not_panic(
        vests in prop::collection::vec(
            (any::<i64>(), any::<u32>(), any::<u16>()),
            0..10
        ),
        milestones in prop::collection::vec(any::<i64>(), 0..10),
        initial_net_worth in any::<i64>(),
        monthly_savings in any::<i64>(),
        months in any::<u16>(),
    ) {
        use logos_core::planning::net_worth_projector::NetWorthProjector;
        use logos_core::planning::fire::UpcomingVest;

        let mut projector = NetWorthProjector::new(initial_net_worth, monthly_savings);
        for milestone in milestones {
            projector.add_milestone_cents(milestone);
        }
        for (price, units, days) in vests {
            projector.add_upcoming_vest(UpcomingVest {
                avg_close_price_cents: price,
                units,
                days_to_vest: days,
            });
        }

        let _ = projector.project_timeline(months);
    }
}

#[test]
fn havoc_posting_credit_min_does_not_panic() {
    let _ =
        logos_core::domain::transaction::Posting::credit(AccountId::new("test").unwrap(), i64::MIN);
}
