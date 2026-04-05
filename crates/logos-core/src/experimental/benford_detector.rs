use crate::domain::transaction::Transaction;

/// Analyzes a set of transactions using Benford's Law to detect potential fraud or synthetic data.
///
/// Benford's Law states that in many naturally occurring collections of numbers, the leading digit
/// is likely to be small. For example, the number 1 appears as the leading digit about 30% of the time,
/// while 9 appears less than 5% of the time.
///
/// This detector calculates the distribution of leading digits of transaction amounts and compares it
/// against the expected Benford distribution using a simplified Chi-Square goodness-of-fit test.
#[derive(Debug, Default)]
pub struct BenfordFraudDetector;

/// Results of a Benford's Law analysis.
#[derive(Debug, Clone, PartialEq)]
pub struct BenfordAnalysisResult {
    /// The actual counts of leading digits 1-9.
    pub digit_counts: [usize; 9],
    /// The actual percentages (0.0 to 1.0) of leading digits 1-9.
    pub actual_distribution: [f64; 9],
    /// The expected percentages (0.0 to 1.0) according to Benford's Law.
    pub expected_distribution: [f64; 9],
    /// The calculated Chi-Square statistic.
    pub chi_square_statistic: f64,
    /// Whether the distribution is flagged as suspicious based on an internal threshold.
    pub is_suspicious: bool,
}

impl BenfordFraudDetector {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Analyzes the given transactions and returns the result.
    ///
    /// Requires at least 10 valid transaction postings to perform a meaningful analysis.
    /// If there is not enough data, returns `None`.
    #[must_use]
    pub fn analyze(&self, transactions: &[Transaction]) -> Option<BenfordAnalysisResult> {
        let mut counts = [0_usize; 9];
        let mut total_count = 0_usize;

        for tx in transactions {
            for posting in tx.postings() {
                let amount = posting.amount().abs();
                if amount > 0 {
                    if let Some(first_digit) = Self::first_digit(amount) {
                        counts[(first_digit - 1) as usize] += 1;
                        total_count += 1;
                    }
                }
            }
        }

        // Require at least 10 data points for a meaningful analysis
        if total_count < 10 {
            return None;
        }

        let mut actual_dist = [0.0; 9];
        for (i, count) in counts.iter().enumerate() {
            #[allow(clippy::cast_precision_loss)]
            let c = *count as f64;
            #[allow(clippy::cast_precision_loss)]
            let t = total_count as f64;
            actual_dist[i] = c / t;
        }

        let expected_dist = [
            0.301, 0.176, 0.125, 0.097, 0.079, 0.067, 0.058, 0.051, 0.046,
        ];

        let mut chi_square = 0.0;
        for i in 0..9 {
            #[allow(clippy::cast_precision_loss)]
            let expected_count = expected_dist[i] * (total_count as f64);

            // Avoid division by zero in extreme edge cases with tiny samples
            if expected_count > 0.0 {
                #[allow(clippy::cast_precision_loss)]
                let actual = counts[i] as f64;
                chi_square += ((actual - expected_count).powi(2)) / expected_count;
            }
        }

        // A threshold of ~15.5 corresponds to a p-value of 0.05 with 8 degrees of freedom.
        // We use a slightly more lenient threshold to avoid false positives on small personal datasets.
        let is_suspicious = chi_square > 20.0;

        Some(BenfordAnalysisResult {
            digit_counts: counts,
            actual_distribution: actual_dist,
            expected_distribution: expected_dist,
            chi_square_statistic: chi_square,
            is_suspicious,
        })
    }

    /// Extracts the first non-zero digit from an integer.
    const fn first_digit(mut num: i64) -> Option<u8> {
        if num == 0 {
            return None;
        }

        while num >= 10 {
            num /= 10;
        }

        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        Some(num as u8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::account::AccountId;
    use crate::domain::transaction::{Posting, TransactionBuilder};

    fn make_tx(amount: i64) -> Transaction {
        TransactionBuilder::new("Test")
            .posting(Posting::credit(AccountId::new("income:salary").unwrap(), amount).unwrap())
            .posting(Posting::debit(AccountId::new("assets:checking").unwrap(), amount).unwrap())
            .build()
            .unwrap()
    }

    #[test]
    fn test_first_digit() {
        assert_eq!(BenfordFraudDetector::first_digit(0), None);
        assert_eq!(BenfordFraudDetector::first_digit(5), Some(5));
        assert_eq!(BenfordFraudDetector::first_digit(123), Some(1));
        assert_eq!(BenfordFraudDetector::first_digit(987_654_321), Some(9));
    }

    #[test]
    fn test_insufficient_data() {
        let detector = BenfordFraudDetector::new();
        let txs = vec![make_tx(100), make_tx(200)]; // Only 4 postings total
        assert!(detector.analyze(&txs).is_none());
    }

    #[test]
    fn test_suspicious_distribution() {
        let detector = BenfordFraudDetector::new();
        let mut txs = Vec::new();

        // Generate highly suspicious data (all starting with 9)
        for _ in 0..20 {
            txs.push(make_tx(9000));
        }

        let result = detector.analyze(&txs).unwrap();

        assert!(result.is_suspicious);
        assert_eq!(result.digit_counts[8], 40); // 20 txs * 2 postings
        assert!(result.chi_square_statistic > 50.0);
    }

    #[test]
    fn test_natural_distribution() {
        let detector = BenfordFraudDetector::new();
        let mut txs = Vec::new();

        // Generate roughly Benford-compliant data
        // 1: ~30%, 2: ~18%, 3: ~12%, etc.
        for _ in 0..30 {
            txs.push(make_tx(1500));
        }
        for _ in 0..18 {
            txs.push(make_tx(2500));
        }
        for _ in 0..12 {
            txs.push(make_tx(3500));
        }
        for _ in 0..10 {
            txs.push(make_tx(4500));
        }
        for _ in 0..8 {
            txs.push(make_tx(5500));
        }
        for _ in 0..7 {
            txs.push(make_tx(6500));
        }
        for _ in 0..6 {
            txs.push(make_tx(7500));
        }
        for _ in 0..5 {
            txs.push(make_tx(8500));
        }
        for _ in 0..4 {
            txs.push(make_tx(9500));
        }

        let result = detector.analyze(&txs).unwrap();

        assert!(!result.is_suspicious);
        assert!(result.chi_square_statistic < 20.0);
    }
}
