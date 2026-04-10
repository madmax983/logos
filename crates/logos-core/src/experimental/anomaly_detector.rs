#![cfg(feature = "nova")]

//! Anomaly Detector
//!
//! Identifies unusually large transactions by account using the Interquartile Range (IQR) method.

use crate::domain::transaction::Transaction;
use std::collections::HashMap;

/// An anomaly detected in the ledger.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Anomaly {
    /// The description of the anomalous transaction.
    pub description: String,
    /// The account where the anomaly occurred.
    pub account: String,
    /// The unusually large amount in cents.
    pub amount_cents: i64,
}

/// Detects unusually large transactions using the IQR method.
#[derive(Debug, Clone)]
pub struct AnomalyDetector {
    multiplier: f64,
}

impl AnomalyDetector {
    /// Creates a new `AnomalyDetector`.
    ///
    /// # Arguments
    /// * `multiplier` - The multiplier for the IQR (typically 1.5 for standard outliers, or 3.0 for extreme outliers).
    #[must_use]
    pub const fn new(multiplier: f64) -> Self {
        Self { multiplier }
    }

    /// Scans a list of transactions and identifies anomalies.
    #[must_use]
    pub fn detect(&self, transactions: &[Transaction]) -> Vec<Anomaly> {
        // Group amounts by account
        let mut account_amounts: HashMap<&str, Vec<i64>> = HashMap::new();

        for tx in transactions {
            let postings = tx.postings();

            // For simplicity, consider debit postings (positive amounts) as expenses.
            for posting in postings {
                if posting.amount() > 0 {
                    account_amounts
                        .entry(posting.account().as_str())
                        .or_default()
                        .push(posting.amount());
                }
            }
        }

        // Calculate IQR bounds for each account
        let mut bounds: HashMap<&str, f64> = HashMap::new();

        for (account, amounts) in &mut account_amounts {
            if amounts.len() < 4 {
                continue; // Not enough data points to reliably calculate IQR
            }

            amounts.sort_unstable();

            let n = amounts.len();
            let mid = n / 2;

            let (lower_half, upper_half) = if n % 2 == 0 {
                (&amounts[0..mid], &amounts[mid..n])
            } else {
                (&amounts[0..mid], &amounts[(mid + 1)..n])
            };

            let q1 = median(lower_half);
            let q3 = median(upper_half);

            let iqr = q3 - q1;
            let upper_bound = self.multiplier.mul_add(iqr, q3);

            bounds.insert(account, upper_bound);
        }

        // Find anomalies
        let mut anomalies = Vec::new();

        for tx in transactions {
            let postings = tx.postings();

            for posting in postings {
                if posting.amount() > 0 {
                    let account_str = posting.account().as_str();

                    if let Some(&upper_bound) = bounds.get(account_str) {
                        #[allow(clippy::cast_precision_loss)]
                        let amount_f64 = posting.amount() as f64;

                        if amount_f64 > upper_bound {
                            anomalies.push(Anomaly {
                                description: tx.description().to_string(),
                                account: account_str.to_string(),
                                amount_cents: posting.amount(),
                            });
                        }
                    }
                }
            }
        }

        anomalies
    }
}

fn median(sorted_data: &[i64]) -> f64 {
    let n = sorted_data.len();
    if n == 0 {
        return 0.0;
    }

    if n % 2 == 0 {
        #[allow(clippy::cast_precision_loss)]
        let m1 = sorted_data[n / 2 - 1] as f64;
        #[allow(clippy::cast_precision_loss)]
        let m2 = sorted_data[n / 2] as f64;
        f64::midpoint(m1, m2)
    } else {
        let mid_val = sorted_data[n / 2];
        #[allow(clippy::cast_precision_loss)]
        let result = mid_val as f64;
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::account::AccountId;
    use crate::domain::transaction::{Posting, TransactionBuilder};

    #[test]
    fn test_detects_anomalies_using_iqr() {
        let detector = AnomalyDetector::new(1.5);
        let mut transactions = Vec::new();

        // Normal amounts: 1000, 1200, 1400, 1500, 2000
        // IQR calculation:
        // Median = 1400
        // Q1 (lower half 1000, 1200) = 1100
        // Q3 (upper half 1500, 2000) = 1750
        // IQR = 1750 - 1100 = 650
        // Upper bound = 1750 + (1.5 * 650) = 2725

        let amounts = vec![1000, 1200, 1400, 1500, 2000];

        for amount in amounts {
            let tx = TransactionBuilder::new("Groceries")
                .posting(
                    Posting::credit(AccountId::new("assets:checking").unwrap(), amount).unwrap(),
                )
                .posting(Posting::debit(AccountId::new("expenses:food").unwrap(), amount).unwrap())
                .build()
                .unwrap();
            transactions.push(tx);
        }

        // Anomalous amount: 3500 (needs to be > 3200 to be an anomaly)
        let tx = TransactionBuilder::new("Fancy Dinner")
            .posting(Posting::credit(AccountId::new("assets:checking").unwrap(), 3500).unwrap())
            .posting(Posting::debit(AccountId::new("expenses:food").unwrap(), 3500).unwrap())
            .build()
            .unwrap();
        transactions.push(tx);

        let anomalies = detector.detect(&transactions);

        assert_eq!(anomalies.len(), 1);
        assert_eq!(anomalies[0].description, "Fancy Dinner");
        assert_eq!(anomalies[0].account, "expenses:food");
        assert_eq!(anomalies[0].amount_cents, 3500);
    }
}
