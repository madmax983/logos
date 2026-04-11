#![allow(clippy::should_panic_without_expect)]

use logos_core::domain::account::AccountId;
#[cfg(feature = "nova")]
use logos_core::domain::category::CategoryGroupId;
use logos_core::domain::rsu::AllocationPolicy;
#[cfg(feature = "nova")]
use logos_core::domain::transaction::{Posting, TransactionBuilder};
use logos_core::experimental::cashflow_projector::{CashflowProjector, RecurringTemplate};
#[cfg(feature = "nova")]
use logos_core::experimental::category_trends::CategoryTrendAnalyzer;
use logos_core::planning::fire::{FireSimulator, UpcomingVest};
use logos_core::planning::net_worth_projector::NetWorthProjector;
use logos_core::planning::rsu_distributor::{RsuAutoDistributor, RsuDistributorConfig};
use proptest::prelude::*;

proptest! {
    #[test]
    #[should_panic(expected = "attempt to add with overflow")]
    fn project_balances_panics_on_overflow(
        start_balance in i64::MAX / 2..i64::MAX,
        amount in i64::MAX / 2..i64::MAX,
    ) {
        let mut projector = CashflowProjector::new();
        projector.set_initial_balance("assets:checking", start_balance);

        projector.add_recurring_template(RecurringTemplate {
            description: "Salary".to_string(),
            amount_cents: amount,
            credit_account: "income:salary".to_string(),
            debit_account: "assets:checking".to_string(),
        });

        // 2 periods will cause the checking balance to exceed i64::MAX because `+=` is unguarded
        let _balances = projector.project_balances(2);
    }

    #[test]
    #[should_panic(expected = "attempt to multiply with overflow")]
    fn distribute_rsu_vest_panics_on_overflow(gross_vest in i64::MAX / 2..i64::MAX) {
        let config = RsuDistributorConfig {
            rsu_asset: AccountId::new("assets:rsu").unwrap(),
            tax_reserve: AccountId::new("assets:tax").unwrap(),
            smoothing_buffer: AccountId::new("assets:buffer").unwrap(),
            goals: AccountId::new("assets:goals").unwrap(),
            discretionary: AccountId::new("assets:checking").unwrap(),
        };
        let distributor = RsuAutoDistributor::new(config);
        let policy = AllocationPolicy::new(40, 20, 30, 10).unwrap();

        // This will panic when multiplied by percent, if gross_vest is very large
        let _ = distributor.distribute_rsu_vest("Vest 1", gross_vest, &policy);
    }

    #[test]
    #[should_panic(expected = "attempt to multiply with overflow")]
    fn project_timeline_panics_on_overflow(
        initial_net_worth in any::<i64>(),
        monthly_savings in any::<i64>(),
        months in 2185..=u16::MAX,
        vest_units in any::<u32>(),
        vest_days in any::<u16>(),
        vest_price in any::<i64>(),
    ) {
        let mut projector = NetWorthProjector::new(initial_net_worth, monthly_savings);
        projector.add_upcoming_vest(UpcomingVest {
            units: vest_units,
            avg_close_price_cents: vest_price,
            days_to_vest: vest_days,
        });

        let _ = projector.project_timeline(months);
    }

    #[test]
    #[should_panic(expected = "attempt to add with overflow")]
    fn add_assets_liabilities_panics_on_overflow(
        assets in i64::MAX / 2..i64::MAX,
    ) {
        let mut sim = FireSimulator::new(5000);
        sim.add_assets_liabilities(assets, 0);
        sim.add_assets_liabilities(assets, 0);
    }

    #[test]
    #[cfg(feature = "nova")]
    #[should_panic(expected = "attempt to add with overflow")]
    fn compute_spending_by_category_panics_on_overflow(
        amount1 in i64::MAX / 2..i64::MAX,
        amount2 in i64::MAX / 2..i64::MAX,
    ) {
        let mut analyzer = CategoryTrendAnalyzer::new();
        let acc = AccountId::new("expenses:misc").unwrap();
        let group = CategoryGroupId::from_name("Misc Group").unwrap();

        analyzer.map_account(acc.clone(), group);

        let tx1 = TransactionBuilder::new("Tx 1")
            .posting(Posting::debit(acc.clone(), amount1).unwrap())
            .posting(Posting::credit(AccountId::new("assets:checking").unwrap(), amount1).unwrap())
            .build()
            .unwrap();

        let tx2 = TransactionBuilder::new("Tx 2")
            .posting(Posting::debit(acc, amount2).unwrap())
            .posting(Posting::credit(AccountId::new("assets:checking").unwrap(), amount2).unwrap())
            .build()
            .unwrap();

        let _trends = analyzer.compute_spending_by_category(&[tx1, tx2]);
    }
}
