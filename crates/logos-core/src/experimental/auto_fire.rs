#![cfg(feature = "nova")]

//! Auto FIRE Simulator
//!
//! Automatically seeds a `FireSimulator` by detecting historical recurring expenses from your ledger.

use crate::domain::transaction::Transaction;
use crate::experimental::recurrence_detector::RecurrenceDetector;
use crate::planning::fire::FireSimulator;

/// Automatically bridges historical transaction data with forward-looking FIRE projections.
#[derive(Debug, Clone)]
pub struct AutoFireSimulator {
    recurrence_detector: RecurrenceDetector,
}

impl AutoFireSimulator {
    /// Creates a new `AutoFireSimulator`.
    ///
    /// # Arguments
    /// * `min_occurrences` - How many times a transaction must appear to be considered a recurring expense.
    #[must_use]
    pub const fn new(min_occurrences: usize) -> Self {
        Self {
            recurrence_detector: RecurrenceDetector::new(min_occurrences),
        }
    }

    /// Scans historical transactions, identifies recurring expenses, sums them to find
    /// your true monthly burn rate, and initializes a `FireSimulator` with that burn rate
    /// and your current liquid assets.
    #[must_use]
    pub fn generate_simulator(
        &self,
        transactions: &[Transaction],
        liquid_assets_cents: i64,
        liabilities_cents: i64,
    ) -> FireSimulator {
        let recurring = self.recurrence_detector.detect(transactions);

        let mut total_monthly_expenses_cents = 0;
        for template in recurring {
            // We only consider it an expense if money is going TO an expense account.
            // In double-entry, a debit to an expense account increases the expense.
            if template.debit_account.starts_with("expenses:") {
                total_monthly_expenses_cents += template.amount_cents;
            }
        }

        let mut fire_sim = FireSimulator::new(total_monthly_expenses_cents);
        fire_sim.add_assets_liabilities(liquid_assets_cents, liabilities_cents);
        fire_sim
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::account::AccountId;
    use crate::domain::transaction::{Posting, TransactionBuilder};

    #[test]
    fn test_auto_fire_simulator_generation() {
        let auto_sim = AutoFireSimulator::new(3);

        let mut transactions = Vec::new();

        // 3 occurrences of Rent ($2000)
        for _ in 0..3 {
            let tx = TransactionBuilder::new("Rent")
                .posting(
                    Posting::credit(AccountId::new("assets:checking").unwrap(), 200_000).unwrap(),
                )
                .posting(
                    Posting::debit(AccountId::new("expenses:housing").unwrap(), 200_000).unwrap(),
                )
                .build()
                .unwrap();
            transactions.push(tx);
        }

        // 3 occurrences of Groceries ($500)
        for _ in 0..3 {
            let tx = TransactionBuilder::new("Groceries")
                .posting(
                    Posting::credit(AccountId::new("assets:checking").unwrap(), 50_000).unwrap(),
                )
                .posting(Posting::debit(AccountId::new("expenses:food").unwrap(), 50_000).unwrap())
                .build()
                .unwrap();
            transactions.push(tx);
        }

        // 3 occurrences of Salary ($5000) - Should be ignored (not an expense)
        for _ in 0..3 {
            let tx = TransactionBuilder::new("Salary")
                .posting(
                    Posting::credit(AccountId::new("income:salary").unwrap(), 500_000).unwrap(),
                )
                .posting(
                    Posting::debit(AccountId::new("assets:checking").unwrap(), 500_000).unwrap(),
                )
                .build()
                .unwrap();
            transactions.push(tx);
        }

        // Generate simulator with $100k assets
        let fire_sim = auto_sim.generate_simulator(&transactions, 10_000_000, 0);

        // Burn rate should be $2500 (Rent + Groceries). 250,000 cents.
        assert_eq!(fire_sim.monthly_expenses_cents(), 250_000);

        // Safe Net Worth should be $100k
        assert_eq!(fire_sim.safe_net_worth_cents(), 10_000_000);
    }
}
