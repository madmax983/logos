//! Synthetic Ledger Generator.
//!
//! Simulates a realistic sequence of double-entry ledger transactions
//! over a period of time based on a financial profile.

use crate::domain::account::AccountId;
use crate::domain::transaction::{Posting, Transaction, TransactionBuilder};

/// A simple Linear Congruential Generator for deterministic randomness.
#[derive(Debug, Clone)]
struct Lcg {
    state: u64,
}

impl Lcg {
    const A: u64 = 6_364_136_223_846_793_005;
    const C: u64 = 1_442_695_040_888_963_407;

    const fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    #[allow(clippy::missing_const_for_fn)]
    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_mul(Self::A).wrapping_add(Self::C);
        self.state
    }

    fn next_f64(&mut self) -> f64 {
        let value = self.next_u64() >> 11;
        #[allow(clippy::cast_precision_loss)]
        let result = value as f64 * (1.0 / (1u64 << 53) as f64);
        result
    }
}

/// The financial profile used to seed the synthetic ledger.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyntheticProfile {
    /// Monthly salary deposited.
    pub monthly_salary_cents: i64,
    /// Monthly rent paid.
    pub monthly_rent_cents: i64,
    /// Base daily spend for discretionary items (e.g., coffee, groceries).
    pub average_daily_spend_cents: i64,
}

/// A generator that builds a realistic historical ledger of transactions.
#[derive(Debug, Clone)]
pub struct SyntheticLedgerGenerator {
    profile: SyntheticProfile,
    seed: u64,
}

impl SyntheticLedgerGenerator {
    /// Creates a new `SyntheticLedgerGenerator`.
    #[must_use]
    pub const fn new(profile: SyntheticProfile, seed: u64) -> Self {
        Self { profile, seed }
    }

    /// Generates transactions simulating `days` of financial activity.
    ///
    /// # Panics
    /// Panics if valid `AccountId`s cannot be parsed (should never happen with these hardcoded prefixes).
    #[must_use]
    pub fn generate(&mut self, days: u32) -> Vec<Transaction> {
        let mut transactions = Vec::new();
        let mut lcg = Lcg::new(self.seed);

        for day in 1..=days {
            // End of month events (approx 30 days)
            if day % 30 == 0 {
                // Receive Salary
                let tx_salary = TransactionBuilder::new(&format!("Salary Day {day}"))
                    .posting(
                        Posting::credit(
                            AccountId::new("income:salary").expect("valid account"),
                            self.profile.monthly_salary_cents,
                        )
                        .unwrap(),
                    )
                    .posting(
                        Posting::debit(
                            AccountId::new("assets:checking").expect("valid account"),
                            self.profile.monthly_salary_cents,
                        )
                        .unwrap(),
                    )
                    .build()
                    .unwrap();
                transactions.push(tx_salary);

                // Pay Rent
                let tx_rent = TransactionBuilder::new(&format!("Rent Day {day}"))
                    .posting(
                        Posting::credit(
                            AccountId::new("assets:checking").expect("valid account"),
                            self.profile.monthly_rent_cents,
                        )
                        .unwrap(),
                    )
                    .posting(
                        Posting::debit(
                            AccountId::new("expenses:rent").expect("valid account"),
                            self.profile.monthly_rent_cents,
                        )
                        .unwrap(),
                    )
                    .build()
                    .unwrap();
                transactions.push(tx_rent);
            }

            // Stochastic daily spending
            // Spend happens with 80% probability
            if lcg.next_f64() < 0.8 {
                // Spend is 50% to 150% of the average daily spend
                let modifier = 0.5 + lcg.next_f64();

                #[allow(clippy::cast_precision_loss)]
                let raw_spend = self.profile.average_daily_spend_cents as f64 * modifier;

                #[allow(clippy::cast_possible_truncation)]
                let spend_cents = raw_spend.round() as i64;

                if spend_cents > 0 {
                    let tx_spend = TransactionBuilder::new(&format!("Daily Spend Day {day}"))
                        .posting(
                            Posting::credit(
                                AccountId::new("assets:checking").expect("valid account"),
                                spend_cents,
                            )
                            .unwrap(),
                        )
                        .posting(
                            Posting::debit(
                                AccountId::new("expenses:discretionary").expect("valid account"),
                                spend_cents,
                            )
                            .unwrap(),
                        )
                        .build()
                        .unwrap();
                    transactions.push(tx_spend);
                }
            }
        }

        // Update generator state for subsequent calls
        self.seed = lcg.next_u64();

        transactions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_synthetic_ledger_generation() {
        let profile = SyntheticProfile {
            monthly_salary_cents: 500_000,
            monthly_rent_cents: 200_000,
            average_daily_spend_cents: 2_000, // $20
        };

        let mut generator = SyntheticLedgerGenerator::new(profile, 42);

        // Simulate 60 days
        let txs = generator.generate(60);

        // 2 salary, 2 rent = 4
        // Daily spends ~ 80% of 60 = ~ 48
        // Total expected ~ 52
        assert!(txs.len() > 40 && txs.len() < 60);

        // Find a salary transaction
        let salary_tx = txs
            .iter()
            .find(|t| t.description().contains("Salary"))
            .unwrap();
        assert_eq!(salary_tx.postings().len(), 2);
    }
}
