#![cfg(feature = "nova")]

//! Portfolio Rebalancer Module
//!
//! Automatically generates double-entry transactions to rebalance a portfolio
//! of assets back to their target allocation percentages.

use crate::domain::account::AccountId;
use crate::domain::transaction::{Posting, Transaction, TransactionBuilder};
use crate::error::DomainError;
use std::collections::HashMap;

/// A target allocation rule defining an asset account and its target percentage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetAllocation {
    /// The asset account
    pub asset: AccountId,
    /// The target percentage (1-100)
    pub percentage: u8,
}

/// Automatically generates rebalancing transactions for a portfolio.
#[derive(Debug, Clone)]
pub struct PortfolioRebalancer {
    targets: Vec<TargetAllocation>,
}

impl PortfolioRebalancer {
    /// Creates a new `PortfolioRebalancer`.
    ///
    /// # Errors
    /// Returns a `DomainError::InvalidAllocationTotal` if the target percentages do not sum exactly to 100.
    pub fn new(targets: Vec<TargetAllocation>) -> Result<Self, DomainError> {
        let total_pct: u32 = targets.iter().map(|t| u32::from(t.percentage)).sum();
        if total_pct != 100 {
            return Err(DomainError::InvalidAllocationTotal {
                total: total_pct.try_into().unwrap_or(u16::MAX),
            });
        }
        Ok(Self { targets })
    }

    /// Generates a transaction to rebalance the current portfolio to the target allocation.
    ///
    /// The `current_balances` map provides the current balance (in cents) for each asset in the target list.
    /// Any fractional cents caused by division are swept into the first target asset to maintain perfect balance.
    ///
    /// # Errors
    /// Returns a `DomainError` if any generated postings are invalid or if the transaction fails to build
    /// (e.g. if the portfolio is already perfectly balanced, it returns `EmptyTransactionPostings`).
    pub fn rebalance(
        &self,
        description: &str,
        current_balances: &HashMap<AccountId, i64>,
    ) -> Result<Transaction, DomainError> {
        let mut total_value: i64 = 0;
        let mut target_values: Vec<(AccountId, i64)> = Vec::with_capacity(self.targets.len());

        for target in &self.targets {
            let balance = current_balances.get(&target.asset).copied().unwrap_or(0);
            total_value += balance;
        }

        if total_value <= 0 {
            // Nothing to rebalance
            return TransactionBuilder::new(description).build();
        }

        let mut remaining_value = total_value;
        for target in &self.targets {
            let allocated = (total_value * i64::from(target.percentage)) / 100;
            target_values.push((target.asset.clone(), allocated));
            remaining_value -= allocated;
        }

        if remaining_value > 0 && !target_values.is_empty() {
            target_values[0].1 += remaining_value;
        }

        let mut builder = TransactionBuilder::new(description);

        for (asset, target_val) in target_values {
            let current_val = current_balances.get(&asset).copied().unwrap_or(0);
            let diff = target_val - current_val;

            if diff > 0 {
                builder = builder.posting(Posting::debit(asset, diff)?);
            } else if diff < 0 {
                builder = builder.posting(Posting::credit(asset, -diff)?);
            }
        }

        builder.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_portfolio_rebalance() {
        let aapl = AccountId::new("assets:aapl").unwrap();
        let tsla = AccountId::new("assets:tsla").unwrap();

        let rebalancer = PortfolioRebalancer::new(vec![
            TargetAllocation {
                asset: aapl.clone(),
                percentage: 60,
            },
            TargetAllocation {
                asset: tsla.clone(),
                percentage: 40,
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
        assert!(postings.contains(&Posting::debit(aapl, 10_000).unwrap()));
        assert!(postings.contains(&Posting::credit(tsla, 10_000).unwrap()));
    }

    #[test]
    fn test_invalid_allocation_total() {
        let aapl = AccountId::new("assets:aapl").unwrap();

        let result = PortfolioRebalancer::new(vec![TargetAllocation {
            asset: aapl,
            percentage: 99,
        }]);

        assert_eq!(
            result.unwrap_err(),
            DomainError::InvalidAllocationTotal { total: 99 }
        );
    }

    #[test]
    fn test_fractional_cents_are_swept_to_first_target() {
        let aapl = AccountId::new("assets:aapl").unwrap();
        let tsla = AccountId::new("assets:tsla").unwrap();
        let msft = AccountId::new("assets:msft").unwrap();

        let rebalancer = PortfolioRebalancer::new(vec![
            TargetAllocation { asset: aapl.clone(), percentage: 33 },
            TargetAllocation { asset: tsla.clone(), percentage: 33 },
            TargetAllocation { asset: msft.clone(), percentage: 34 },
        ]).unwrap();

        let mut current_balances = std::collections::HashMap::new();
        current_balances.insert(aapl.clone(), 101);

        let tx = rebalancer.rebalance("Rebalance", &current_balances).unwrap();
        let postings = tx.postings();

        assert_eq!(postings.len(), 3);
        assert!(postings.contains(&Posting::credit(aapl.clone(), 67).unwrap()));
        assert!(postings.contains(&Posting::debit(tsla.clone(), 33).unwrap()));
        assert!(postings.contains(&Posting::debit(msft.clone(), 34).unwrap()));
    }

    #[test]
    fn test_perfectly_balanced_asset_is_ignored() {
        let aapl = AccountId::new("assets:aapl").unwrap();
        let tsla = AccountId::new("assets:tsla").unwrap();
        let msft = AccountId::new("assets:msft").unwrap();

        let rebalancer = PortfolioRebalancer::new(vec![
            TargetAllocation { asset: aapl.clone(), percentage: 50 },
            TargetAllocation { asset: tsla.clone(), percentage: 30 },
            TargetAllocation { asset: msft.clone(), percentage: 20 },
        ]).unwrap();

        let mut current_balances = std::collections::HashMap::new();
        current_balances.insert(aapl.clone(), 40_000);
        current_balances.insert(tsla.clone(), 40_000);
        current_balances.insert(msft.clone(), 20_000);

        let tx = rebalancer.rebalance("Rebalance", &current_balances).unwrap();
        let postings = tx.postings();

        assert_eq!(postings.len(), 2);
        assert!(postings.contains(&Posting::debit(aapl.clone(), 10_000).unwrap()));
        assert!(postings.contains(&Posting::credit(tsla.clone(), 10_000).unwrap()));
    }

    #[test]
    fn test_empty_targets_with_remaining_value() {
        let aapl = AccountId::new("assets:aapl").unwrap();
        let tsla = AccountId::new("assets:tsla").unwrap();

        let rebalancer = PortfolioRebalancer::new(vec![
            TargetAllocation { asset: aapl.clone(), percentage: 50 },
            TargetAllocation { asset: tsla.clone(), percentage: 50 },
        ]).unwrap();

        let mut current_balances = std::collections::HashMap::new();
        current_balances.insert(aapl.clone(), 50);
        current_balances.insert(tsla.clone(), 50);

        let tx = rebalancer.rebalance("Rebalance", &current_balances);
        assert!(matches!(tx.unwrap_err(), DomainError::EmptyTransactionPostings));
    }
}
