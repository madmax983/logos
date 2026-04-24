#![allow(clippy::should_panic_without_expect)]
use logos_core::AccountId;
use logos_core::portfolio_rebalancer::{PortfolioRebalancer, TargetAllocation};
use proptest::prelude::*;
use std::collections::HashMap;

proptest! {
    #[test]
    #[should_panic]
    fn portfolio_rebalancer_panics_on_overflow(
        val1 in (i64::MAX / 2) + 1..=i64::MAX,
        val2 in (i64::MAX / 2) + 1..=i64::MAX
    ) {
        let aapl = AccountId::new("assets:aapl").unwrap();
        let tsla = AccountId::new("assets:tsla").unwrap();
        let rebalancer = PortfolioRebalancer::new(vec![
            TargetAllocation { asset: aapl.clone(), percentage: 50 },
            TargetAllocation { asset: tsla.clone(), percentage: 50 },
        ]).unwrap();

        let mut balances = HashMap::new();
        balances.insert(aapl, val1);
        balances.insert(tsla, val2);

        let _ = rebalancer.rebalance("Rebalance", &balances);
    }
}
