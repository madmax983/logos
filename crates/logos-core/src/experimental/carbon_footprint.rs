use crate::domain::transaction::Transaction;
use std::collections::HashMap;

/// Estimates carbon footprint based on transaction expenses.
///
/// Associates expense accounts with CO2 emission factors (kg CO2 per dollar).
#[derive(Debug, Default)]
pub struct CarbonFootprintEstimator {
    factors: HashMap<String, f64>,
}

impl CarbonFootprintEstimator {
    /// Creates a new `CarbonFootprintEstimator`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds an emission factor for a specific account prefix.
    pub fn add_factor(&mut self, account_prefix: &str, kg_co2_per_dollar: f64) {
        self.factors
            .insert(account_prefix.to_owned(), kg_co2_per_dollar);
    }

    /// Estimates total CO2 emissions for a slice of transactions.
    #[must_use]
    pub fn estimate(&self, transactions: &[Transaction]) -> f64 {
        let mut total_kg_co2 = 0.0;

        for tx in transactions {
            for posting in tx.postings() {
                // Only consider debits (expenses)
                if posting.amount() > 0 {
                    let account = posting.account().as_str();
                    // Find matching factor
                    for (prefix, factor) in &self.factors {
                        if account.starts_with(prefix) {
                            // Convert cents to dollars
                            #[allow(clippy::cast_precision_loss)]
                            let amount_dollars = posting.amount() as f64 / 100.0;
                            total_kg_co2 += amount_dollars * factor;
                        }
                    }
                }
            }
        }

        total_kg_co2
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AccountId;
    use crate::domain::transaction::{Posting, TransactionBuilder};

    #[test]
    fn test_estimate() {
        let mut estimator = CarbonFootprintEstimator::new();
        estimator.add_factor("expenses:gas", 0.05);

        let tx = TransactionBuilder::new("Gas Station")
            .posting(Posting::debit(AccountId::new("expenses:gas").unwrap(), 2000).unwrap())
            .posting(Posting::credit(AccountId::new("assets:checking").unwrap(), 2000).unwrap())
            .build()
            .unwrap();

        // 20.00 * 0.05 = 1.0 kg CO2
        assert!((estimator.estimate(&[tx]) - 1.0).abs() < f64::EPSILON);
    }
}
