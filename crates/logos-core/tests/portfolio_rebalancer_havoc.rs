#![allow(clippy::should_panic_without_expect)]
use logos_core::domain::account::AccountId;
use logos_core::experimental::portfolio_rebalancer::{PortfolioRebalancer, TargetAllocation};
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

#[test]
fn sentinel_test_rebalance_remaining_value_exact_zero() {
    let aapl = AccountId::new("assets:aapl").unwrap();
    let tsla = AccountId::new("assets:tsla").unwrap();

    let rebalancer = PortfolioRebalancer::new(vec![
        TargetAllocation {
            asset: aapl.clone(),
            percentage: 50,
        },
        TargetAllocation {
            asset: tsla.clone(),
            percentage: 50,
        },
    ])
    .unwrap();

    let mut current_balances = HashMap::new();
    // 50 + 50 = 100. target aapl = 100*0.5=50. target tsla = 100*0.5=50. remaining = 0
    current_balances.insert(aapl, 50);
    current_balances.insert(tsla, 50);

    let tx = rebalancer
        .rebalance("Rebalance", &current_balances)
        .unwrap_err();
    assert_eq!(tx, logos_core::error::DomainError::EmptyTransactionPostings);
}

#[test]
fn sentinel_test_rebalance_needs_credit_and_debit() {
    let aapl = AccountId::new("assets:aapl").unwrap();
    let tsla = AccountId::new("assets:tsla").unwrap();

    let rebalancer = PortfolioRebalancer::new(vec![
        TargetAllocation {
            asset: aapl.clone(),
            percentage: 50,
        },
        TargetAllocation {
            asset: tsla.clone(),
            percentage: 50,
        },
    ])
    .unwrap();

    let mut current_balances = HashMap::new();
    current_balances.insert(aapl.clone(), 100_000);
    current_balances.insert(tsla.clone(), 0);

    let tx = rebalancer
        .rebalance("Rebalance", &current_balances)
        .unwrap();
    let postings = tx.postings();

    assert_eq!(postings.len(), 2);
    // aapl has 100,000. total value 100,000. target 50,000. aapl diff = 50,000 - 100,000 = -50,000 -> credit 50,000
    // tsla has 0. total value 100,000. target 50,000. tsla diff = 50,000 - 0 = +50,000 -> debit 50,000
    assert!(
        postings.contains(&logos_core::domain::transaction::Posting::credit(aapl, 50_000).unwrap())
    );
    assert!(
        postings.contains(&logos_core::domain::transaction::Posting::debit(tsla, 50_000).unwrap())
    );
}

#[test]
fn sentinel_test_rebalance_credit_only() {
    let aapl = AccountId::new("assets:aapl").unwrap();
    let tsla = AccountId::new("assets:tsla").unwrap();

    let rebalancer = PortfolioRebalancer::new(vec![
        TargetAllocation {
            asset: aapl.clone(),
            percentage: 100,
        },
        TargetAllocation {
            asset: tsla.clone(),
            percentage: 0,
        },
    ])
    .unwrap();

    let mut current_balances = HashMap::new();
    current_balances.insert(aapl.clone(), 50_000);
    current_balances.insert(tsla.clone(), 50_000);

    let tx = rebalancer
        .rebalance("Rebalance", &current_balances)
        .unwrap();
    let postings = tx.postings();

    assert_eq!(postings.len(), 2);
    // aapl has 50k. tsla has 50k. Total is 100k. Target aapl is 100k. target tsla is 0
    // aapl diff: 100k - 50k = 50k -> debit 50k
    // tsla diff: 0 - 50k = -50k -> credit 50k
    assert!(
        postings.contains(&logos_core::domain::transaction::Posting::debit(aapl, 50_000).unwrap())
    );
    assert!(
        postings.contains(&logos_core::domain::transaction::Posting::credit(tsla, 50_000).unwrap())
    );
}

#[test]
fn sentinel_test_rebalance_exact_amounts() {
    let aapl = AccountId::new("assets:aapl").unwrap();
    let tsla = AccountId::new("assets:tsla").unwrap();

    let rebalancer = PortfolioRebalancer::new(vec![
        TargetAllocation {
            asset: aapl.clone(),
            percentage: 50,
        },
        TargetAllocation {
            asset: tsla.clone(),
            percentage: 50,
        },
    ])
    .unwrap();

    let mut current_balances = HashMap::new();
    current_balances.insert(aapl.clone(), 60_000);
    current_balances.insert(tsla.clone(), 40_000);

    let tx = rebalancer
        .rebalance("Rebalance", &current_balances)
        .unwrap();
    let postings = tx.postings();
    assert_eq!(postings.len(), 2);
    // total: 100k
    // target aapl: 50k, target tsla: 50k
    // aapl diff: 50k - 60k = -10k -> credit 10k
    // tsla diff: 50k - 40k = 10k -> debit 10k
    assert!(
        postings.contains(&logos_core::domain::transaction::Posting::credit(aapl, 10_000).unwrap())
    );
    assert!(
        postings.contains(&logos_core::domain::transaction::Posting::debit(tsla, 10_000).unwrap())
    );
}
