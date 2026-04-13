#![allow(missing_docs)]
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
    /// Creates a new `RecurrenceDetector` with the specified threshold.
    #[must_use]
    pub const fn new(min_occurrences: usize) -> Self {
        Self { min_occurrences }
    }

    /// Scans a list of transactions and identifies recurring templates.
    ///
    /// ⚡ Bolt Optimization: Uses borrowed strings `(&str)` for the temporary grouping hash map
    /// to avoid 3 heap allocations (description, credit, debit strings) per transaction
    /// analyzed in the hot path.
    #[must_use]
    pub fn detect(&self, transactions: &[Transaction]) -> Vec<RecurringTemplate> {
        // ⚡ Bolt: Group by references (&str) instead of owned Strings to avoid
        // 3 heap allocations per transaction during the aggregation phase.
        // Group by (description, credit_account, debit_account, amount)
        let mut groups: HashMap<(&str, &str, &str, i64), usize> = HashMap::new();

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
                    tx.description(),
                    c.account().as_str(),
                    d.account().as_str(),
                    d.amount(), // the positive amount
                );
                *groups.entry(key).or_insert(0) += 1;
            }
        }

        let mut templates = Vec::new();
        for ((description, credit_account, debit_account, amount_cents), count) in groups {
            if count >= self.min_occurrences {
                templates.push(RecurringTemplate {
                    description: description.to_owned(),
                    amount_cents,
                    credit_account: credit_account.to_owned(),
                    debit_account: debit_account.to_owned(),
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
