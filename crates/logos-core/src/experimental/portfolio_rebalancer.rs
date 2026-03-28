use crate::domain::account::AccountId;
use crate::domain::transaction::{Posting, Transaction, TransactionBuilder};
use std::collections::HashMap;

/// Defines the target percentage allocation for a specific account.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetAllocation {
    pub account: AccountId,
    /// The target allocation percentage (e.g., 60 for 60%).
    pub target_weight: u32,
}

/// Automatically calculates double-entry transactions to rebalance a portfolio
/// based on current balances and a target percentage allocation.
#[derive(Debug, Clone)]
pub struct PortfolioRebalancer {
    allocations: Vec<TargetAllocation>,
}

impl PortfolioRebalancer {
    /// Creates a new `PortfolioRebalancer` with the specified target allocations.
    ///
    /// # Errors
    ///
    /// Returns an error if the total target weights do not sum to exactly 100%.
    pub fn new(allocations: Vec<TargetAllocation>) -> Result<Self, &'static str> {
        let total: u32 = allocations.iter().map(|a| a.target_weight).sum();
        if total != 100 {
            return Err("Target weights must sum to exactly 100");
        }
        Ok(Self { allocations })
    }

    /// Calculates the exact transactions needed to rebalance the portfolio.
    ///
    /// Generates a single transaction moving funds from overweight accounts
    /// to underweight accounts to match the target allocations.
    ///
    /// # Errors
    ///
    /// Returns an error if generating the rebalance transaction fails.
    pub fn rebalance(
        &self,
        current_balances: &HashMap<AccountId, i64>,
    ) -> Result<Option<Transaction>, &'static str> {
        let mut total_balance: i64 = 0;
        for balance in current_balances.values() {
            total_balance = total_balance
                .checked_add(*balance)
                .ok_or("Balance overflow")?;
        }

        if total_balance <= 0 {
            return Ok(None);
        }

        let mut deltas: HashMap<AccountId, i64> = HashMap::new();

        for alloc in &self.allocations {
            let current = current_balances.get(&alloc.account).copied().unwrap_or(0);

            #[allow(clippy::cast_precision_loss)]
            let target_f64 = (total_balance as f64) * f64::from(alloc.target_weight) / 100.0;

            #[allow(clippy::cast_possible_truncation)]
            let target = target_f64.round() as i64;

            let diff = target - current;
            if diff != 0 {
                deltas.insert(alloc.account.clone(), diff);
            }
        }

        if deltas.is_empty() {
            return Ok(None);
        }

        let mut builder = TransactionBuilder::new("Portfolio Rebalance");
        let mut net_diff: i64 = 0;

        for (account, diff) in &deltas {
            net_diff += diff;
            if *diff > 0 {
                let posting =
                    Posting::debit(account.clone(), *diff).map_err(|_| "Invalid debit")?;
                builder = builder.posting(posting);
            } else if *diff < 0 {
                let posting =
                    Posting::credit(account.clone(), diff.abs()).map_err(|_| "Invalid credit")?;
                builder = builder.posting(posting);
            }
        }

        // Handle penny rounding issues from percentages
        if net_diff != 0 {
            return Err("Rounding error caused unbalanced rebalance");
        }

        let tx = builder.build().map_err(|_| "Failed to build transaction")?;
        Ok(Some(tx))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_portfolio_rebalancer() {
        let mut balances = HashMap::new();
        let stocks = AccountId::new("assets:stocks").unwrap();
        let bonds = AccountId::new("assets:bonds").unwrap();

        balances.insert(stocks.clone(), 800_000); // 80%
        balances.insert(bonds.clone(), 200_000); // 20%

        let rebalancer = PortfolioRebalancer::new(vec![
            TargetAllocation {
                account: stocks.clone(),
                target_weight: 60,
            },
            TargetAllocation {
                account: bonds.clone(),
                target_weight: 40,
            },
        ])
        .unwrap();

        let tx = rebalancer.rebalance(&balances).unwrap().unwrap();

        let postings = tx.postings();
        assert_eq!(postings.len(), 2);

        let stock_posting = postings.iter().find(|p| p.account() == &stocks).unwrap();
        assert_eq!(stock_posting.amount(), -200_000);

        let bond_posting = postings.iter().find(|p| p.account() == &bonds).unwrap();
        assert_eq!(bond_posting.amount(), 200_000);
    }
}
