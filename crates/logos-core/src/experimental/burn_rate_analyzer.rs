use crate::domain::transaction::Transaction;

/// Analyzes historical transactions to calculate average burn rate and runway.
#[derive(Debug, Clone)]
pub struct BurnRateAnalyzer {
    transactions: Vec<Transaction>,
}

impl BurnRateAnalyzer {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            transactions: Vec::new(),
        }
    }

    /// Adds a transaction to the analyzer.
    pub fn add_transaction(&mut self, transaction: Transaction) {
        self.transactions.push(transaction);
    }

    /// Calculates the average monthly burn rate (expenses) and the remaining runway
    /// in months given a current liquid asset balance.
    ///
    /// Assumes all transactions provided occurred over `months_analyzed`.
    #[must_use]
    pub fn calculate_runway(
        &self,
        current_balance_cents: i64,
        months_analyzed: u16,
    ) -> Option<RunwayMetrics> {
        if months_analyzed == 0 || self.transactions.is_empty() {
            return None;
        }

        let mut total_expenses_cents = 0_i64;

        for tx in &self.transactions {
            for posting in tx.postings() {
                // A debit to an expense account is an expense.
                if posting.amount() > 0 && posting.account().as_str().starts_with("expenses:") {
                    total_expenses_cents = total_expenses_cents.saturating_add(posting.amount());
                }
            }
        }

        if total_expenses_cents == 0 {
            return None; // Infinite runway or no expenses
        }

        let average_monthly_burn = total_expenses_cents / i64::from(months_analyzed);

        let runway_months = if average_monthly_burn > 0 {
            current_balance_cents / average_monthly_burn
        } else {
            return None;
        };

        Some(RunwayMetrics {
            average_monthly_burn_cents: average_monthly_burn,
            runway_months: runway_months.try_into().unwrap_or(u16::MAX),
        })
    }
}

impl Default for BurnRateAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Metrics describing burn rate and runway.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunwayMetrics {
    pub average_monthly_burn_cents: i64,
    pub runway_months: u16,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::account::AccountId;
    use crate::domain::transaction::{Posting, TransactionBuilder};

    #[test]
    fn test_burn_rate_calculation() {
        let mut analyzer = BurnRateAnalyzer::new();

        let tx1 = TransactionBuilder::new("Rent")
            .posting(Posting::debit(AccountId::new("expenses:rent").unwrap(), 150_000).unwrap())
            .posting(Posting::credit(AccountId::new("assets:checking").unwrap(), 150_000).unwrap())
            .build()
            .unwrap();

        let tx2 = TransactionBuilder::new("Groceries")
            .posting(Posting::debit(AccountId::new("expenses:food").unwrap(), 50_000).unwrap())
            .posting(Posting::credit(AccountId::new("assets:checking").unwrap(), 50_000).unwrap())
            .build()
            .unwrap();

        let tx3 = TransactionBuilder::new("Income")
            .posting(Posting::credit(AccountId::new("income:salary").unwrap(), 500_000).unwrap())
            .posting(Posting::debit(AccountId::new("assets:checking").unwrap(), 500_000).unwrap())
            .build()
            .unwrap();

        analyzer.add_transaction(tx1);
        analyzer.add_transaction(tx2);
        analyzer.add_transaction(tx3); // Income should be ignored

        // Total expenses = $2000. Over 2 months = $1000/mo.
        // Balance = $12000. Runway = 12 months.
        let metrics = analyzer.calculate_runway(1_200_000, 2).unwrap();

        assert_eq!(metrics.average_monthly_burn_cents, 100_000);
        assert_eq!(metrics.runway_months, 12);
    }
}
