//! Transaction Anomaly Detector
//!
//! A statistical tool that analyzes past transaction postings and flags
//! specific transactions that deviate significantly from historical spending patterns
//! for the same account. Uses Z-score calculations.

use std::collections::HashMap;

use crate::domain::account::AccountId;
use crate::domain::transaction::Transaction;

/// An anomaly detection result for a single posting.
#[derive(Debug, Clone, PartialEq)]
pub struct Anomaly {
    pub account: AccountId,
    pub amount: i64,
    pub z_score: f64,
}

/// Anomaly Detector that flags unusual transaction amounts.
#[derive(Debug, Default)]
pub struct AnomalyDetector {
    // We'll store a history of amounts for each account.
    history: HashMap<AccountId, Vec<i64>>,
}

impl AnomalyDetector {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Feeds historical transactions into the detector to build a baseline.
    pub fn train(&mut self, transaction: &Transaction) {
        for posting in transaction.postings() {
            let entry = self
                .history
                .entry(posting.account().clone())
                .or_default();
            entry.push(posting.amount());
        }
    }

    /// Evaluates a transaction against the historical baseline.
    /// Returns a list of anomalies if any posting exceeds the threshold Z-score.
    #[must_use]
    pub fn evaluate(&self, transaction: &Transaction, threshold_z: f64) -> Vec<Anomaly> {
        let mut anomalies = Vec::new();

        for posting in transaction.postings() {
            if let Some(history) = self.history.get(posting.account()) {
                // We need at least 2 data points for standard deviation to make sense
                if history.len() < 2 {
                    continue;
                }

                #[allow(clippy::cast_precision_loss)]
                let n = history.len() as f64;

                let sum: i64 = history.iter().sum();
                #[allow(clippy::cast_precision_loss)]
                let mean = sum as f64 / n;

                let variance: f64 = history
                    .iter()
                    .map(|&val| {
                        #[allow(clippy::cast_precision_loss)]
                        let diff = val as f64 - mean;
                        diff * diff
                    })
                    // Uses population variance. Could use `n - 1.0` for sample variance.
                    .sum::<f64>()
                    / n;

                let std_dev = variance.sqrt();

                // To prevent division by zero or overly sensitive flags if all past amounts are exactly the same
                if std_dev < 0.01 {
                    continue;
                }

                #[allow(clippy::cast_precision_loss)]
                let z_score = (posting.amount() as f64 - mean).abs() / std_dev;

                if z_score > threshold_z {
                    anomalies.push(Anomaly {
                        account: posting.account().clone(),
                        amount: posting.amount(),
                        z_score,
                    });
                }
            }
        }

        anomalies
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::transaction::TransactionBuilder;

    #[test]
    fn should_flag_anomalous_spending() {
        let mut detector = AnomalyDetector::new();
        let account = AccountId::new("expenses:food").unwrap();
        let checking = AccountId::new("assets:checking").unwrap();

        // Train on $100 grocery runs, with a little variation so std_dev != 0
        for i in 0..10 {
            let offset = (i - 5) * 100; // -500 to +400 variation
            let amount = 10000 + offset;
            let t = TransactionBuilder::new("Groceries")
                .posting(
                    crate::domain::transaction::Posting::debit(account.clone(), amount).unwrap(),
                )
                .posting(
                    crate::domain::transaction::Posting::credit(checking.clone(), amount).unwrap(),
                )
                .build()
                .unwrap();
            detector.train(&t);
        }

        // Evaluate a $500 grocery run
        let target = TransactionBuilder::new("Huge Groceries")
            .posting(crate::domain::transaction::Posting::debit(account.clone(), 50000).unwrap())
            .posting(crate::domain::transaction::Posting::credit(checking, 50000).unwrap())
            .build()
            .unwrap();

        let anomalies = detector.evaluate(&target, 2.0); // Z-score > 2.0 is anomalous
        assert_eq!(anomalies.len(), 2);
        assert_eq!(anomalies[0].account, account);
        assert!(anomalies[0].z_score > 2.0);
    }
}
