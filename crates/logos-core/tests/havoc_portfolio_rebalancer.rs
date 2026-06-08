use logos_core::portfolio_rebalancer::{PortfolioRebalancer, TargetAllocation};
use logos_core::AccountId;
use std::collections::HashMap;

#[test]
#[should_panic(expected = "attempt to subtract with overflow")]
fn test_havoc_portfolio_rebalancer_panics_on_overflow() {
    let aapl = AccountId::new("assets:aapl").unwrap();
    let tsla = AccountId::new("assets:tsla").unwrap();
    let msft = AccountId::new("assets:msft").unwrap();

    let rebalancer = PortfolioRebalancer::new(vec![
        TargetAllocation {
            asset: aapl.clone(),
            percentage: 33,
        },
        TargetAllocation {
            asset: tsla.clone(),
            percentage: 33,
        },
        TargetAllocation {
            asset: msft.clone(),
            percentage: 34,
        },
    ]).unwrap();

    let mut current_balances: HashMap<AccountId, i64> = HashMap::new();
    current_balances.insert(aapl.clone(), -184467440737095517);
    current_balances.insert(tsla.clone(), 3319304228375684979);
    current_balances.insert(msft.clone(), 6088535249216186345);

    let _ = rebalancer.rebalance("Rebalance", &current_balances);
}
