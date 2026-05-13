#![cfg(feature = "nova")]

//! Rebalancing Income Router Module
//!
//! Automatically routes income (like a paycheck) into various destination accounts,
//! specifically directing new funds toward underweight assets to naturally drift
//! a portfolio back to its target allocation without triggering taxable sell events.

use crate::domain::account::AccountId;
use crate::domain::transaction::{Posting, Transaction, TransactionBuilder};
use crate::error::DomainError;
use crate::experimental::portfolio_rebalancer::TargetAllocation;
use std::collections::HashMap;

/// Automatically directs new income toward underweight portfolio assets.
#[derive(Debug, Clone)]
pub struct RebalancingIncomeRouter {
    source_account: AccountId,
    targets: Vec<TargetAllocation>,
}

impl RebalancingIncomeRouter {
    /// Creates a new `RebalancingIncomeRouter`.
    ///
    /// # Errors
    /// Returns a `DomainError::InvalidAllocationTotal` if the target percentages do not sum exactly to 100.
    pub fn new(
        source_account: AccountId,
        targets: Vec<TargetAllocation>,
    ) -> Result<Self, DomainError> {
        let total_pct: u32 = targets.iter().map(|t| u32::from(t.percentage)).sum();
        if total_pct != 100 {
            return Err(DomainError::InvalidAllocationTotal {
                total: total_pct.try_into().unwrap_or(u16::MAX),
            });
        }
        Ok(Self {
            source_account,
            targets,
        })
    }

    /// Routes the income amount, creating a perfectly balanced transaction.
    ///
    /// It calculates the new total portfolio value, then the target value for each asset.
    /// It directs the new income entirely toward the assets that are furthest below their targets.
    ///
    /// # Errors
    /// Returns a `DomainError::InvalidCreditAmount` if the `amount_cents` is zero or negative.
    /// Returns a `DomainError` if the underlying transaction builder fails.
    pub fn route_income(
        &self,
        description: &str,
        amount_cents: i64,
        current_balances: &HashMap<AccountId, i64>,
    ) -> Result<Transaction, DomainError> {
        if amount_cents <= 0 {
            return Err(DomainError::InvalidCreditAmount {
                amount: amount_cents,
            });
        }

        let mut current_total: i64 = 0;
        let mut actual_balances = HashMap::new();

        for target in &self.targets {
            let bal = current_balances.get(&target.asset).copied().unwrap_or(0);
            actual_balances.insert(target.asset.clone(), bal);
            current_total = current_total.saturating_add(bal);
        }

        let future_total = current_total.saturating_add(amount_cents);

        // Calculate deficits
        let mut deficits = Vec::new();
        for target in &self.targets {
            let target_value = future_total.saturating_mul(i64::from(target.percentage)) / 100;
            let current_value = actual_balances.get(&target.asset).copied().unwrap_or(0);

            if target_value > current_value {
                deficits.push((target.asset.clone(), target_value - current_value));
            }
        }

        // Sort deficits descending, so we fund the most underweight assets first
        deficits.sort_by(|a, b| b.1.cmp(&a.1));

        let mut builder = TransactionBuilder::new(description)
            .posting(Posting::credit(self.source_account.clone(), amount_cents)?);

        let mut remaining_to_allocate = amount_cents;

        for (asset, deficit) in deficits {
            if remaining_to_allocate == 0 {
                break;
            }

            let allocation = remaining_to_allocate.min(deficit);
            builder = builder.posting(Posting::debit(asset, allocation)?);
            remaining_to_allocate -= allocation;
        }

        // If there's still money left (e.g., portfolio is balanced and we have extra),
        // distribute based on target percentages
        if remaining_to_allocate > 0 {
            let mut allocations = Vec::with_capacity(self.targets.len());
            let mut remainder = remaining_to_allocate;

            for target in &self.targets {
                let alloc =
                    remaining_to_allocate.saturating_mul(i64::from(target.percentage)) / 100;
                allocations.push((target.asset.clone(), alloc));
                remainder -= alloc;
            }

            if remainder > 0 && !allocations.is_empty() {
                allocations[0].1 += remainder;
            }

            for (asset, alloc) in allocations {
                if alloc > 0 {
                    builder = builder.posting(Posting::debit(asset, alloc)?);
                }
            }
        }

        builder.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_route_income_to_underweight_asset() {
        let source = AccountId::new("income:salary").unwrap();
        let eq_acc = AccountId::new("assets:equities").unwrap();
        let bond_acc = AccountId::new("assets:bonds").unwrap();

        let router = RebalancingIncomeRouter::new(
            source,
            vec![
                TargetAllocation {
                    asset: eq_acc.clone(),
                    percentage: 60,
                },
                TargetAllocation {
                    asset: bond_acc.clone(),
                    percentage: 40,
                },
            ],
        )
        .unwrap();

        // Currently heavily skewed to equities ($8000 eq, $1000 bonds)
        let mut balances = HashMap::new();
        balances.insert(eq_acc.clone(), 8000);
        balances.insert(bond_acc.clone(), 1000);

        // Add $1000 income.
        // Future total = $10,000. Target: $6000 eq, $4000 bonds.
        // Equities is OVER target (8000 > 6000). Bonds is UNDER target (1000 < 4000).
        // ALL $1000 should go to bonds.
        let tx = router.route_income("Paycheck", 1000, &balances).unwrap();
        let postings = tx.postings();

        assert_eq!(postings.len(), 2);
        assert!(postings.contains(&Posting::debit(bond_acc, 1000).unwrap()));
    }

    #[test]
    fn test_route_income_perfectly_balanced() {
        let source = AccountId::new("income:salary").unwrap();
        let eq_acc = AccountId::new("assets:equities").unwrap();
        let bond_acc = AccountId::new("assets:bonds").unwrap();

        let router = RebalancingIncomeRouter::new(
            source,
            vec![
                TargetAllocation {
                    asset: eq_acc.clone(),
                    percentage: 60,
                },
                TargetAllocation {
                    asset: bond_acc.clone(),
                    percentage: 40,
                },
            ],
        )
        .unwrap();

        // Currently perfectly balanced ($6000 eq, $4000 bonds)
        let mut balances = HashMap::new();
        balances.insert(eq_acc.clone(), 6000);
        balances.insert(bond_acc.clone(), 4000);

        // Add $1000 income.
        // Because it's perfectly balanced, the $1000 should distribute 60/40.
        let tx = router.route_income("Paycheck", 1000, &balances).unwrap();
        let postings = tx.postings();

        assert_eq!(postings.len(), 3);
        assert!(postings.contains(&Posting::debit(eq_acc, 600).unwrap()));
        assert!(postings.contains(&Posting::debit(bond_acc, 400).unwrap()));
    }

    #[test]
    fn test_route_income_zero() {
        let source = AccountId::new("income:salary").unwrap();
        let eq_acc = AccountId::new("assets:equities").unwrap();

        let router = RebalancingIncomeRouter::new(
            source,
            vec![TargetAllocation {
                asset: eq_acc.clone(),
                percentage: 100,
            }],
        )
        .unwrap();

        let balances = HashMap::new();
        assert_eq!(
            router.route_income("Paycheck", 0, &balances).unwrap_err(),
            DomainError::InvalidCreditAmount { amount: 0 }
        );
    }
}
