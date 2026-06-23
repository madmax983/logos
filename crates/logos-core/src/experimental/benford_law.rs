use crate::domain::transaction::Transaction;
use std::collections::HashMap;

/// Analyzes transaction amounts against Benford's Law distribution.
///
/// Benford's Law states that in many naturally occurring collections of numbers,
/// the leading significant digit is likely to be small. This is often used in
/// forensic accounting to detect potential fraud or anomalies.
#[derive(Debug, Default)]
pub struct BenfordLawAnalyzer {
    digit_counts: HashMap<u8, usize>,
    total_samples: usize,
}

impl BenfordLawAnalyzer {
    /// Initializes an empty analyzer ready to detect statistical anomalies.
    ///
    /// Benford's law requires a significant amount of transaction volume to be
    /// statistically sound. Create this once, feed it thousands of transactions,
    /// and then observe the distribution.
    ///
    /// # Examples
    ///
    /// ```
    /// use logos_core::experimental::benford_law::BenfordLawAnalyzer;
    /// use logos_core::domain::transaction::{TransactionBuilder, Posting};
    /// use logos_core::AccountId;
    ///
    /// let mut analyzer = BenfordLawAnalyzer::new();
    /// let tx = TransactionBuilder::new("T1")
    ///     .posting(Posting::credit(AccountId::new("income").unwrap(), 100).unwrap())
    ///     .posting(Posting::debit(AccountId::new("checking").unwrap(), 100).unwrap())
    ///     .build()
    ///     .unwrap();
    ///
    /// analyzer.add_transactions(&[tx]);
    /// let observed = analyzer.observed_distribution();
    /// assert_eq!(observed.get(&1).copied(), Some(1.0)); // 100 starts with 1
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds transactions to the analyzer.
    pub fn add_transactions(&mut self, transactions: &[Transaction]) {
        for tx in transactions {
            for posting in tx.postings() {
                let amount = posting.amount().abs();
                if amount > 0 {
                    if let Some(first_digit) = Self::extract_first_digit(amount) {
                        *self.digit_counts.entry(first_digit).or_insert(0) += 1;
                        self.total_samples += 1;
                    }
                }
            }
        }
    }

    const fn extract_first_digit(mut number: i64) -> Option<u8> {
        if number == 0 {
            return None;
        }
        while number >= 10 {
            number /= 10;
        }
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        Some(number as u8)
    }

    /// Computes the observed distribution of the first digits (1-9).
    #[must_use]
    pub fn observed_distribution(&self) -> HashMap<u8, f64> {
        let mut dist = HashMap::new();
        if self.total_samples == 0 {
            return dist;
        }

        for d in 1..=9 {
            let count = self.digit_counts.get(&d).copied().unwrap_or(0);
            #[allow(clippy::cast_precision_loss)]
            let percent = (count as f64) / (self.total_samples as f64);
            dist.insert(d, percent);
        }
        dist
    }

    /// Returns the theoretical expected distribution according to Benford's Law.
    #[must_use]
    pub fn expected_distribution() -> HashMap<u8, f64> {
        let mut dist = HashMap::new();
        for d in 1..=9 {
            let p = f64::log10(1.0 + (1.0 / f64::from(d)));
            dist.insert(d, p);
        }
        dist
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AccountId;
    use crate::domain::transaction::{Posting, TransactionBuilder};

    #[test]
    fn test_extract_first_digit() {
        assert_eq!(BenfordLawAnalyzer::extract_first_digit(0), None);
        assert_eq!(BenfordLawAnalyzer::extract_first_digit(5), Some(5));
        assert_eq!(BenfordLawAnalyzer::extract_first_digit(1234), Some(1));
        assert_eq!(BenfordLawAnalyzer::extract_first_digit(999), Some(9));
    }

    #[test]
    fn test_benford_law_analysis() {
        let mut analyzer = BenfordLawAnalyzer::new();
        let tx1 = TransactionBuilder::new("T1")
            .posting(Posting::credit(AccountId::new("income").unwrap(), 100).unwrap())
            .posting(Posting::debit(AccountId::new("checking").unwrap(), 100).unwrap())
            .build()
            .unwrap();

        let tx2 = TransactionBuilder::new("T2")
            .posting(Posting::credit(AccountId::new("income").unwrap(), 250).unwrap())
            .posting(Posting::debit(AccountId::new("checking").unwrap(), 250).unwrap())
            .build()
            .unwrap();

        analyzer.add_transactions(&[tx1, tx2]);

        let observed = analyzer.observed_distribution();

        // 100 and 100 start with 1, 250 and 250 start with 2
        // total 4 postings: two '1's, two '2's
        assert_eq!(observed.get(&1).copied(), Some(0.5));
        assert_eq!(observed.get(&2).copied(), Some(0.5));
        assert_eq!(observed.get(&3).copied(), Some(0.0));
    }

    #[test]
    fn test_expected_distribution() {
        let expected = BenfordLawAnalyzer::expected_distribution();
        assert!((expected[&1] - 0.301).abs() < 0.01);
        assert!((expected[&9] - 0.045).abs() < 0.01);
    }
}
