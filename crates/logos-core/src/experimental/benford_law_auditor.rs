use std::collections::HashMap;

use crate::domain::transaction::Transaction;

/// Auditor that applies Benford's Law to transaction amounts to detect potential
/// anomalies, fabricated data, or systemic errors.
///
/// Benford's Law states that in many naturally occurring collections of numbers,
/// the leading digit is likely to be small. For example, '1' appears as the leading
/// digit about 30% of the time. This auditor compares the ledger's actual distribution
/// against the theoretical distribution.
#[derive(Debug, Default)]
pub struct BenfordLawAuditor {
    transactions: Vec<Transaction>,
}

/// The result of a Benford's Law analysis.
#[derive(Debug, PartialEq)]
pub struct BenfordAnalysis {
    /// The observed probability distribution of leading digits (1-9).
    pub observed_frequencies: HashMap<u8, f64>,
    /// The theoretical probability distribution according to Benford's Law.
    pub theoretical_frequencies: HashMap<u8, f64>,
    /// The calculated Chi-square statistic indicating deviation from the expected distribution.
    pub chi_square_statistic: f64,
    /// A flag indicating if the data strongly deviates from Benford's Law.
    pub is_anomalous: bool,
}

impl BenfordLawAuditor {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a transaction to the auditor's dataset.
    pub fn add_transaction(&mut self, transaction: Transaction) {
        self.transactions.push(transaction);
    }

    /// Analyzes the accumulated transactions and returns the Benford's Law statistics.
    ///
    /// Returns `None` if there are no non-zero postings to analyze.
    #[must_use]
    pub fn analyze(&self) -> Option<BenfordAnalysis> {
        let mut digit_counts: HashMap<u8, usize> = HashMap::new();
        let mut total_amounts = 0;

        for tx in &self.transactions {
            for posting in tx.postings() {
                let amount = posting.amount().abs();
                if amount == 0 {
                    continue;
                }

                let mut num = amount;
                while num >= 10 {
                    num /= 10;
                }

                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                let first_digit = num as u8;

                if first_digit > 0 && first_digit <= 9 {
                    *digit_counts.entry(first_digit).or_insert(0) += 1;
                    total_amounts += 1;
                }
            }
        }

        if total_amounts == 0 {
            return None;
        }

        let mut observed_frequencies = HashMap::new();
        let mut chi_square_statistic = 0.0;
        let mut theoretical_frequencies = HashMap::new();

        for digit in 1..=9 {
            let count = *digit_counts.get(&digit).unwrap_or(&0);

            #[allow(clippy::cast_precision_loss)]
            let observed_freq = count as f64 / f64::from(total_amounts);
            observed_frequencies.insert(digit, observed_freq);

            let expected_freq = (1.0 + 1.0 / f64::from(digit)).log10();
            theoretical_frequencies.insert(digit, expected_freq);

            let expected_count = expected_freq * f64::from(total_amounts);

            if expected_count > 0.0 {
                #[allow(clippy::cast_precision_loss)]
                let diff = (count as f64) - expected_count;
                chi_square_statistic += (diff * diff) / expected_count;
            }
        }

        // A simple threshold for anomaly detection.
        // For 8 degrees of freedom, the critical value for p=0.05 is 15.507.
        let is_anomalous = chi_square_statistic > 15.507;

        Some(BenfordAnalysis {
            observed_frequencies,
            theoretical_frequencies,
            chi_square_statistic,
            is_anomalous,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AccountId;
    use crate::domain::transaction::TransactionBuilder;

    #[test]
    fn test_empty_analysis() {
        let auditor = BenfordLawAuditor::new();
        assert_eq!(auditor.analyze(), None);
    }

    #[test]
    fn test_benford_compliance() {
        let mut auditor = BenfordLawAuditor::new();

        // Create transactions that roughly follow Benford's law for leading digits
        // 1: 30%, 2: 17%, 3: 12%, etc.
        let amounts = vec![
            100, 150, 1200, 199, 10, 13, // 6 with leading 1
            200, 250, 29, // 3 with leading 2
            300, 399, // 2 with leading 3
            450, // 1 with leading 4
            500, // 1 with leading 5
            600, // 1 with leading 6
            700, // 1 with leading 7
            800, // 1 with leading 8
            900, // 1 with leading 9
        ];

        for amount in amounts {
            let tx = TransactionBuilder::new("Test")
                .posting(
                    crate::domain::transaction::Posting::credit(
                        AccountId::new("income:salary").unwrap(),
                        amount,
                    )
                    .unwrap(),
                )
                .posting(
                    crate::domain::transaction::Posting::debit(
                        AccountId::new("assets:checking").unwrap(),
                        amount,
                    )
                    .unwrap(),
                )
                .build()
                .unwrap();
            auditor.add_transaction(tx);
        }

        let analysis = auditor.analyze().unwrap();

        // This set is very small, but shouldn't be statistically anomalous enough to fail
        assert!(
            !analysis.is_anomalous,
            "Should not be anomalous. Chi2: {}",
            analysis.chi_square_statistic
        );
        assert_eq!(analysis.theoretical_frequencies.len(), 9);
        assert_eq!(analysis.observed_frequencies.len(), 9);
    }

    #[test]
    fn test_benford_anomaly() {
        let mut auditor = BenfordLawAuditor::new();

        // Create transactions that completely violate Benford's law (all start with 9)
        let amounts = vec![900, 950, 999, 90, 9200, 91, 98, 970];

        for amount in amounts {
            let tx = TransactionBuilder::new("Test")
                .posting(
                    crate::domain::transaction::Posting::credit(
                        AccountId::new("income:salary").unwrap(),
                        amount,
                    )
                    .unwrap(),
                )
                .posting(
                    crate::domain::transaction::Posting::debit(
                        AccountId::new("assets:checking").unwrap(),
                        amount,
                    )
                    .unwrap(),
                )
                .build()
                .unwrap();
            auditor.add_transaction(tx);
        }

        let analysis = auditor.analyze().unwrap();

        // Everything starts with 9, this is highly anomalous
        assert!(analysis.is_anomalous);
    }
}
