#![cfg(feature = "nova")]

//! Income Router Module
//!
//! Automatically routes income (like a paycheck) into various destination accounts based on percentage rules.

use crate::domain::account::AccountId;
use crate::domain::transaction::{Posting, Transaction, TransactionBuilder};
use crate::error::DomainError;

/// A routing rule defining a destination and a percentage allocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteRule {
    /// The destination account
    pub destination: AccountId,
    /// The percentage to allocate (1-100)
    pub percentage: u8,
}

/// Automatically routes a single income amount to multiple destinations.
#[derive(Debug, Clone)]
pub struct IncomeRouter {
    source_account: AccountId,
    rules: Vec<RouteRule>,
}

impl IncomeRouter {
    /// Creates a new `IncomeRouter`.
    ///
    /// # Errors
    /// Returns a `DomainError::InvalidAllocationTotal` if the percentages do not sum exactly to 100.
    pub fn new(source_account: AccountId, rules: Vec<RouteRule>) -> Result<Self, DomainError> {
        let total_pct: u32 = rules.iter().map(|r| u32::from(r.percentage)).sum();
        if total_pct != 100 {
            return Err(DomainError::InvalidAllocationTotal {
                total: total_pct.try_into().unwrap_or(u16::MAX),
            });
        }
        Ok(Self {
            source_account,
            rules,
        })
    }

    /// Routes the income amount, creating a perfectly balanced transaction.
    /// Any fractional cents are swept into the first rule's destination.
    ///
    /// # Errors
    /// Returns a `DomainError::InvalidCreditAmount` if the `amount_cents` is zero or negative.
    /// Returns a `DomainError` if the underlying transaction builder fails.
    pub fn route_income(
        &self,
        description: &str,
        amount_cents: i64,
    ) -> Result<Transaction, DomainError> {
        if amount_cents <= 0 {
            return Err(DomainError::InvalidCreditAmount {
                amount: amount_cents,
            });
        }

        let mut builder = TransactionBuilder::new(description)
            .posting(Posting::credit(self.source_account.clone(), amount_cents)?);

        let mut remaining_cents = amount_cents;
        let mut allocations = Vec::with_capacity(self.rules.len());

        // Calculate exact allocations, leaving remainders
        for rule in &self.rules {
            let allocated = (amount_cents * i64::from(rule.percentage)) / 100;
            allocations.push((rule.destination.clone(), allocated));
            remaining_cents -= allocated;
        }

        // Sweep remainder to the first bucket (if any)
        if remaining_cents > 0 {
            allocations[0].1 += remaining_cents;
        }

        for (dest, amount) in allocations {
            if amount > 0 {
                builder = builder.posting(Posting::debit(dest, amount)?);
            }
        }

        builder.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_income_routing() {
        let source = AccountId::new("income:salary").unwrap();
        let dest1 = AccountId::new("assets:checking").unwrap();
        let dest2 = AccountId::new("assets:savings").unwrap();

        let router = IncomeRouter::new(
            source,
            vec![
                RouteRule {
                    destination: dest1,
                    percentage: 60,
                },
                RouteRule {
                    destination: dest2,
                    percentage: 40,
                },
            ],
        )
        .unwrap();

        let tx = router.route_income("Paycheck", 1000).unwrap();
        let postings = tx.postings();

        assert_eq!(postings.len(), 3);
        assert!(
            postings.contains(
                &Posting::credit(AccountId::new("income:salary").unwrap(), 1000).unwrap()
            )
        );
        assert!(
            postings.contains(
                &Posting::debit(AccountId::new("assets:checking").unwrap(), 600).unwrap()
            )
        );
        assert!(
            postings
                .contains(&Posting::debit(AccountId::new("assets:savings").unwrap(), 400).unwrap())
        );
    }

    #[test]
    fn test_invalid_allocation_total() {
        let source = AccountId::new("income:salary").unwrap();
        let dest1 = AccountId::new("assets:checking").unwrap();

        let result = IncomeRouter::new(
            source,
            vec![RouteRule {
                destination: dest1,
                percentage: 99,
            }],
        );

        assert_eq!(
            result.unwrap_err(),
            DomainError::InvalidAllocationTotal { total: 99 }
        );
    }

    #[test]
    fn test_sweep_remainder() {
        let source = AccountId::new("income:salary").unwrap();
        let dest1 = AccountId::new("assets:checking").unwrap();
        let dest2 = AccountId::new("assets:savings").unwrap();
        let dest3 = AccountId::new("expenses:tax").unwrap();

        let router = IncomeRouter::new(
            source,
            vec![
                RouteRule {
                    destination: dest1.clone(),
                    percentage: 33,
                },
                RouteRule {
                    destination: dest2.clone(),
                    percentage: 33,
                },
                RouteRule {
                    destination: dest3.clone(),
                    percentage: 34,
                },
            ],
        )
        .unwrap();

        // 10 cents. 33% = 3, 33% = 3, 34% = 3. Remaining = 1. Goes to first bucket (checking).
        // Checking: 4. Savings: 3. Tax: 3.
        let tx = router.route_income("Paycheck", 10).unwrap();
        let postings = tx.postings();

        assert!(postings.contains(&Posting::debit(dest1, 4).unwrap()));
        assert!(postings.contains(&Posting::debit(dest2, 3).unwrap()));
        assert!(postings.contains(&Posting::debit(dest3, 3).unwrap()));
    }

    #[test]
    fn test_zero_amount_allocation_is_skipped() {
        let source = AccountId::new("income:salary").unwrap();
        let dest1 = AccountId::new("assets:checking").unwrap();
        let dest2 = AccountId::new("assets:savings").unwrap();

        let router = IncomeRouter::new(
            source,
            vec![
                RouteRule {
                    destination: dest1.clone(),
                    percentage: 99,
                },
                RouteRule {
                    destination: dest2,
                    percentage: 1,
                },
            ],
        )
        .unwrap();

        // 1% of 40 cents is 0.4 cents, which truncates to 0 cents.
        // It should skip dest2 and put all 40 cents in dest1 (39 + 1 remainder).
        let tx = router.route_income("Paycheck", 40).unwrap();
        let postings = tx.postings();

        assert_eq!(postings.len(), 2);
        assert!(postings.contains(&Posting::debit(dest1, 40).unwrap()));
    }
}
