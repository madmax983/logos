#![allow(clippy::should_panic_without_expect)]
use logos_core::AccountId;
use logos_core::portfolio_rebalancer::{PortfolioRebalancer, TargetAllocation};
use std::collections::HashMap;

#[test]
#[should_panic]
fn havoc_portfolio_rebalancer_overflow() {
    let aapl = AccountId::new("assets:aapl").unwrap();
    let tsla = AccountId::new("assets:tsla").unwrap();
    let msft = AccountId::new("assets:msft").unwrap();
    let rebalancer2 = PortfolioRebalancer::new(vec![
        TargetAllocation {
            asset: aapl.clone(),
            percentage: 100,
        },
        TargetAllocation {
            asset: tsla.clone(),
            percentage: 0,
        },
        TargetAllocation {
            asset: msft.clone(),
            percentage: 0,
        },
    ])
    .unwrap();

    let mut current_balances = HashMap::new();
    current_balances.insert(aapl, i64::MIN);
    current_balances.insert(tsla, i64::MAX);
    current_balances.insert(msft, 10);

    let _ = rebalancer2.rebalance("Rebalance", &current_balances);
}
