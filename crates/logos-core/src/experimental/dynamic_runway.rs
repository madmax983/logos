#![cfg(feature = "nova")]

//! Dynamic Runway Analyzer
//!
//! A module that computes an empirical monthly burn rate from a set of historical
//! transactions and feeds it into the `RunwaySimulator` to project true financial runway.

use crate::domain::transaction::Transaction;
use crate::experimental::runway_simulator::RunwaySimulator;

/// A report detailing the dynamic runway calculation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DynamicRunwayReport {
    /// The empirically calculated average monthly burn in cents.
    pub empirical_monthly_burn_cents: i64,
    /// The number of months the assets will last based on the simulation.
    pub runway_months: u32,
    /// Total amount expected to be burned in cents over the runway period.
    pub total_expected_burn_cents: i64,
}

/// Analyzes historical transactions to determine empirical runway.
#[derive(Debug, Clone)]
pub struct DynamicRunwayAnalyzer {
    liquid_assets_cents: i64,
    annual_inflation_pct: f64,
}

impl DynamicRunwayAnalyzer {
    /// Creates a new `DynamicRunwayAnalyzer`.
    #[must_use]
    pub const fn new(liquid_assets_cents: i64, annual_inflation_pct: f64) -> Self {
        Self {
            liquid_assets_cents,
            annual_inflation_pct,
        }
    }

    /// Analyzes a list of historical transactions to compute the average monthly burn rate,
    /// then calculates the financial runway using that empirical burn rate.
    ///
    /// It assumes the provided transactions represent exactly `history_months` of spending.
    /// Only debits (positive amounts) to expense accounts (typically those prefixed with "expenses:")
    /// are counted towards the burn rate.
    #[must_use]
    pub fn analyze(
        &self,
        transactions: &[Transaction],
        history_months: u16,
    ) -> DynamicRunwayReport {
        if history_months == 0 {
            return DynamicRunwayReport {
                empirical_monthly_burn_cents: 0,
                runway_months: 1200,
                total_expected_burn_cents: 0,
            };
        }

        let mut total_spend: i64 = 0;

        for tx in transactions {
            for posting in tx.postings() {
                // We assume burn comes from positive debits to expense accounts.
                // A more advanced version might take a list of specific account IDs.
                if posting.amount() > 0 && posting.account().as_str().starts_with("expenses:") {
                    total_spend += posting.amount();
                }
            }
        }

        let empirical_monthly_burn_cents = total_spend / i64::from(history_months);

        let simulator = RunwaySimulator::new(
            self.liquid_assets_cents,
            empirical_monthly_burn_cents,
            self.annual_inflation_pct,
        );

        let result = simulator.calculate_runway();

        DynamicRunwayReport {
            empirical_monthly_burn_cents,
            runway_months: result.months,
            total_expected_burn_cents: result.total_burned_cents,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::account::AccountId;
    use crate::domain::transaction::{Posting, TransactionBuilder};

    #[test]
    fn test_dynamic_runway_calculation() {
        let analyzer = DynamicRunwayAnalyzer::new(12_000_000, 0.0); // $120k liquid assets, 0% inflation

        let mut transactions = Vec::new();

        // Month 1: $3k rent, $1k food
        let tx1 = TransactionBuilder::new("Rent")
            .posting(Posting::debit(AccountId::new("expenses:rent").unwrap(), 300_000).unwrap())
            .posting(Posting::credit(AccountId::new("assets:checking").unwrap(), 300_000).unwrap())
            .build()
            .unwrap();
        transactions.push(tx1);

        let tx2 = TransactionBuilder::new("Groceries")
            .posting(Posting::debit(AccountId::new("expenses:food").unwrap(), 100_000).unwrap())
            .posting(Posting::credit(AccountId::new("assets:checking").unwrap(), 100_000).unwrap())
            .build()
            .unwrap();
        transactions.push(tx2);

        // Month 2: $3k rent, $1k food
        let tx3 = TransactionBuilder::new("Rent")
            .posting(Posting::debit(AccountId::new("expenses:rent").unwrap(), 300_000).unwrap())
            .posting(Posting::credit(AccountId::new("assets:checking").unwrap(), 300_000).unwrap())
            .build()
            .unwrap();
        transactions.push(tx3);

        let tx4 = TransactionBuilder::new("Groceries")
            .posting(Posting::debit(AccountId::new("expenses:food").unwrap(), 100_000).unwrap())
            .posting(Posting::credit(AccountId::new("assets:checking").unwrap(), 100_000).unwrap())
            .build()
            .unwrap();
        transactions.push(tx4);

        // Total spend over 2 months is $8k. Average is $4k / month.
        let report = analyzer.analyze(&transactions, 2);

        assert_eq!(report.empirical_monthly_burn_cents, 400_000);
        // $120k / $4k = 30 months
        assert_eq!(report.runway_months, 30);
        assert_eq!(report.total_expected_burn_cents, 12_000_000);
    }

    #[test]
    fn test_dynamic_runway_zero_months() {
        let analyzer = DynamicRunwayAnalyzer::new(10_000_000, 0.0);
        let report = analyzer.analyze(&[], 0);

        assert_eq!(report.empirical_monthly_burn_cents, 0);
        assert_eq!(report.runway_months, 1200);
    }
}
