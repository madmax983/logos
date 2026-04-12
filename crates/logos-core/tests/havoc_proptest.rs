#![allow(clippy::should_panic_without_expect)]

use logos_core::domain::account::AccountId;
use logos_core::domain::rsu::AllocationPolicy;
use logos_core::experimental::cashflow_projector::{CashflowProjector, RecurringTemplate};
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

    #[cfg(feature = "nova")]
    #[test]
    #[should_panic(expected = "attempt to add with overflow")]
    fn compute_spending_by_category_panics_on_overflow(
        amount in i64::MAX / 2..i64::MAX,
    ) {
        use logos_core::experimental::category_trends::CategoryTrendAnalyzer;
        use logos_core::domain::category::CategoryGroupId;
        use logos_core::domain::transaction::{TransactionBuilder, Posting};

        let mut analyzer = CategoryTrendAnalyzer::new();
        let group_id = CategoryGroupId::from_name("food").unwrap();
        let acc_id = AccountId::new("expenses:food").unwrap();
        analyzer.map_account(acc_id.clone(), group_id);

        let posting1 = Posting::debit(acc_id, amount).unwrap();
        let posting2 = Posting::credit(AccountId::new("assets:checking").unwrap(), amount).unwrap();

        let tx = TransactionBuilder::new("Test").posting(posting1).posting(posting2).build().unwrap();
        let txs = vec![tx.clone(), tx];

        let _ = analyzer.compute_spending_by_category(&txs);
    }

    #[cfg(feature = "nova")]
    #[test]
    #[should_panic(expected = "attempt to multiply with overflow")]
    fn simulate_debt_optimizer_panics_on_overflow(
        balance in i64::MAX / 2..i64::MAX,
    ) {
        use logos_core::experimental::debt_optimizer::{DebtOptimizer, Debt, PayoffStrategy};

        let mut optimizer = DebtOptimizer::new(100_000);
        optimizer.add_debt(Debt {
            name: "Massive Debt".to_string(),
            balance_cents: balance,
            interest_rate_pct: 15,
            min_payment_cents: 50_000,
        });

        let _ = optimizer.simulate(PayoffStrategy::Avalanche);
    }
}
