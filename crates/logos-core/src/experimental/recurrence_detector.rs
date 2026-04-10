//! The Pattern Whisperer
//!
//! Automated cashflow recurrence detection.
//!
//! Manual budget forecasting is tedious and error-prone. Users often forget recurring subscriptions,
//! utility bills, or irregular income streams. The [`RecurrenceDetector`] analyzes historical
//! transactions to deterministically identify these patterns, surfacing them to reduce the
//! cognitive load of month-over-month planning.

use crate::domain::transaction::Transaction;
use crate::experimental::cashflow_projector::RecurringTemplate;
use std::collections::HashMap;

/// Detects recurring transactions from a historical ledger.
///
/// It scans a list of transactions, looking for those with exactly two postings
/// (one debit, one credit). If the same transaction (matching description,
/// credit account, debit account, and amount) occurs at least `min_occurrences`
/// times, it generates a `RecurringTemplate`.
#[derive(Debug, Clone)]
pub struct RecurrenceDetector {
    min_occurrences: usize,
}

impl RecurrenceDetector {
    /// Initializes a recurrence detector with a sensitivity threshold.
    ///
    /// The threshold dictates how many identical occurrences of a transaction must be
    /// found before it is considered a recurring pattern.
    ///
    /// ## Examples
    ///
    /// ```
    /// use logos_core::experimental::recurrence_detector::RecurrenceDetector;
    ///
    /// // Require at least 3 identical transactions to establish a pattern
    /// let detector = RecurrenceDetector::new(3);
    /// ```
    #[must_use]
    pub const fn new(min_occurrences: usize) -> Self {
        Self { min_occurrences }
    }

    /// Scans a ledger history to identify recurring cashflow patterns.
    ///
    /// Filters the provided `transactions` for simple, two-posting transfers (one debit, one credit).
    /// If an identical transfer (matching description, source account, destination account, and amount)
    /// occurs at least `min_occurrences` times, a [`RecurringTemplate`] is generated.
    ///
    /// ## Examples
    ///
    /// ```
    /// use logos_core::experimental::recurrence_detector::RecurrenceDetector;
    /// use logos_core::domain::transaction::{TransactionBuilder, Posting};
    /// use logos_core::AccountId;
    ///
    /// let mut transactions = Vec::new();
    /// let account_checking = AccountId::new("assets:checking").unwrap();
    /// let account_netflix = AccountId::new("expenses:netflix").unwrap();
    ///
    /// // Add 3 identical transactions
    /// for _ in 0..3 {
    ///     transactions.push(
    ///         TransactionBuilder::new("Netflix Subscription")
    ///             .posting(Posting::credit(account_checking.clone(), 1599).unwrap())
    ///             .posting(Posting::debit(account_netflix.clone(), 1599).unwrap())
    ///             .build()
    ///             .unwrap()
    ///     );
    /// }
    ///
    /// let detector = RecurrenceDetector::new(3);
    /// let templates = detector.detect(&transactions);
    ///
    /// assert_eq!(templates.len(), 1);
    /// assert_eq!(templates[0].description, "Netflix Subscription");
    /// assert_eq!(templates[0].amount_cents, 1599);
    /// ```
    #[must_use]
    pub fn detect(&self, transactions: &[Transaction]) -> Vec<RecurringTemplate> {
        // Group by (description, credit_account, debit_account, amount)
        let mut groups: HashMap<(String, String, String, i64), usize> = HashMap::new();

        for tx in transactions {
            let postings = tx.postings();

            // Only consider simple transactions with exactly one debit and one credit.
            if postings.len() != 2 {
                continue;
            }

            let mut credit = None;
            let mut debit = None;

            for posting in postings {
                if posting.amount() < 0 {
                    credit = Some(posting);
                } else if posting.amount() > 0 {
                    debit = Some(posting);
                }
            }

            if let (Some(c), Some(d)) = (credit, debit) {
                let key = (
                    tx.description().to_owned(),
                    c.account().as_str().to_owned(),
                    d.account().as_str().to_owned(),
                    d.amount(), // the positive amount
                );
                *groups.entry(key).or_insert(0) += 1;
            }
        }

        let mut templates = Vec::new();
        for ((description, credit_account, debit_account, amount_cents), count) in groups {
            if count >= self.min_occurrences {
                templates.push(RecurringTemplate {
                    description,
                    amount_cents,
                    credit_account,
                    debit_account,
                });
            }
        }

        templates.sort_unstable_by(|a, b| a.description.cmp(&b.description));
        templates
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::account::AccountId;
    use crate::domain::transaction::{Posting, TransactionBuilder};

    #[test]
    fn test_detects_recurring_transactions() {
        let detector = RecurrenceDetector::new(3);

        let mut transactions = Vec::new();

        // 3 occurrences of a recurring transaction
        for _ in 0..3 {
            let tx = TransactionBuilder::new("Netflix")
                .posting(Posting::credit(AccountId::new("assets:checking").unwrap(), 1599).unwrap())
                .posting(
                    Posting::debit(AccountId::new("expenses:entertainment").unwrap(), 1599)
                        .unwrap(),
                )
                .build()
                .unwrap();
            transactions.push(tx);
        }

        // 1 occurrence of a one-off transaction
        let tx = TransactionBuilder::new("Coffee")
            .posting(Posting::credit(AccountId::new("assets:checking").unwrap(), 500).unwrap())
            .posting(Posting::debit(AccountId::new("expenses:food").unwrap(), 500).unwrap())
            .build()
            .unwrap();
        transactions.push(tx);

        let templates = detector.detect(&transactions);

        assert_eq!(templates.len(), 1);
        assert_eq!(templates[0].description, "Netflix");
        assert_eq!(templates[0].amount_cents, 1599);
        assert_eq!(templates[0].credit_account, "assets:checking");
        assert_eq!(templates[0].debit_account, "expenses:entertainment");
    }
}
