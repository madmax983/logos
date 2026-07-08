#![cfg(feature = "nova")]

//! Auto Invest Router Module
//!
//! Automatically routes incoming funds into a portfolio by directing the money
//! towards underweight assets, moving the overall portfolio closer to its
//! target allocation without needing to sell existing assets.

use crate::domain::account::AccountId;
use crate::domain::transaction::{Posting, Transaction, TransactionBuilder};
use crate::error::DomainError;
use crate::experimental::portfolio_rebalancer::TargetAllocation;
use std::collections::HashMap;

/// Automatically routes income to underweight assets to reach target allocations.
#[derive(Debug, Clone)]
pub struct AutoInvestRouter {
    targets: Vec<TargetAllocation>,
}

impl AutoInvestRouter {
    /// Creates a new `AutoInvestRouter`.
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

    /// Generates a transaction that invests `amount_cents` by routing it to the most underweight assets.
    ///
    /// The `current_balances` map provides the current balance (in cents) for each asset in the target list.
    ///
    /// # Errors
    /// Returns a `DomainError` if the amount is negative or zero, or if the transaction fails to build.
    pub fn route_investment(
        &self,
        description: &str,
        amount_cents: i64,
        source_account: AccountId,
        current_balances: &HashMap<AccountId, i64>,
    ) -> Result<Transaction, DomainError> {
        if amount_cents <= 0 {
            return Err(DomainError::InvalidCreditAmount {
                amount: amount_cents,
            });
        }

        let mut current_total: i64 = 0;
        for target in &self.targets {
            let balance = current_balances.get(&target.asset).copied().unwrap_or(0);
            current_total = current_total.saturating_add(balance);
        }

        let future_total = current_total.saturating_add(amount_cents);

        let mut gaps = Vec::new();
        let mut total_gap: i64 = 0;

        for target in &self.targets {
            let current_balance = current_balances.get(&target.asset).copied().unwrap_or(0);
            let ideal_balance = future_total.saturating_mul(i64::from(target.percentage)) / 100;
            let gap = ideal_balance - current_balance;
            if gap > 0 {
                gaps.push((target.asset.clone(), gap));
                total_gap += gap;
            }
        }

        let mut allocations = Vec::new();
        let mut remaining = amount_cents;

        if total_gap > 0 {
            for (asset, gap) in &gaps {
                // Determine how much of the investment should go to this gap
                #[allow(clippy::cast_precision_loss)]
                let proportion = *gap as f64 / total_gap as f64;
                #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
                let allocate = (amount_cents as f64 * proportion).round() as i64;

                let actual_allocate = allocate.min(remaining);
                if actual_allocate > 0 {
                    allocations.push((asset.clone(), actual_allocate));
                    remaining -= actual_allocate;
                }
            }
        }

        // If there's remaining money (due to rounding, or if there were no gaps because
        // the portfolio perfectly matches targets but total_gap was 0), sweep to the first target.
        if remaining > 0 {
            if let Some(target) = self.targets.first() {
                if let Some(pos) = allocations.iter().position(|(a, _)| a == &target.asset) {
                    allocations[pos].1 += remaining;
                } else {
                    allocations.push((target.asset.clone(), remaining));
                }
            }
        }

        let mut builder = TransactionBuilder::new(description)
            .posting(Posting::credit(source_account, amount_cents)?);

        for (asset, amount) in allocations {
            builder = builder.posting(Posting::debit(asset, amount)?);
        }

        builder.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto_invest_routing_to_underweight() {
        let source = AccountId::new("assets:checking").unwrap();
        let aapl = AccountId::new("assets:aapl").unwrap();
        let tsla = AccountId::new("assets:tsla").unwrap();

        // Target: 50% AAPL, 50% TSLA
        let router = AutoInvestRouter::new(vec![
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

        // Current: $60 in AAPL, $20 in TSLA. Total = $80.
        // We want to invest $20. Future total = $100.
        // Ideal: $50 AAPL, $50 TSLA.
        // Gaps: AAPL has $60 (gap -10, no gap), TSLA has $20 (gap 30).
        // The entire $20 should go to TSLA.
        let mut balances = HashMap::new();
        balances.insert(aapl, 6000);
        balances.insert(tsla.clone(), 2000);

        let tx = router
            .route_investment("Invest", 2000, source.clone(), &balances)
            .unwrap();
        let postings = tx.postings();

        assert_eq!(postings.len(), 2);
        assert!(postings.contains(&Posting::credit(source, 2000).unwrap()));
        assert!(postings.contains(&Posting::debit(tsla, 2000).unwrap()));
    }

    #[test]
    fn test_auto_invest_split_proportional() {
        let source = AccountId::new("assets:checking").unwrap();
        let aapl = AccountId::new("assets:aapl").unwrap();
        let tsla = AccountId::new("assets:tsla").unwrap();

        // Target: 50% AAPL, 50% TSLA
        let router = AutoInvestRouter::new(vec![
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

        // Current: $0 in AAPL, $0 in TSLA.
        // Invest $100. Should split $50/$50.
        let balances = HashMap::new();

        let tx = router
            .route_investment("Invest", 10000, source.clone(), &balances)
            .unwrap();
        let postings = tx.postings();

        assert_eq!(postings.len(), 3);
        assert!(postings.contains(&Posting::credit(source, 10000).unwrap()));
        assert!(postings.contains(&Posting::debit(aapl, 5000).unwrap()));
        assert!(postings.contains(&Posting::debit(tsla, 5000).unwrap()));
    }
}
