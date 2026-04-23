use crate::domain::account::AccountId;
use crate::domain::transaction::{Posting, Transaction, TransactionBuilder};
use crate::error::DomainError;

/// Simulates a micro-investing "round-up" feature.
///
/// This feature monitors transactions. When money is spent from a `source_account`
/// (i.e., a credit posting on that account), it identifies expenses (debits) that are
/// not perfectly round dollars, calculates the spare change needed to reach the next
/// full dollar, and constructs a new transaction moving that spare change from the
/// source account to a `savings_account`.
#[derive(Debug)]
pub struct RoundUpSimulator {
    source_account: AccountId,
    savings_account: AccountId,
}

impl RoundUpSimulator {
    /// Creates a new `RoundUpSimulator`.
    #[must_use]
    pub const fn new(source_account: AccountId, savings_account: AccountId) -> Self {
        Self {
            source_account,
            savings_account,
        }
    }

    /// Processes a transaction and returns a round-up transaction if applicable.
    ///
    /// # Errors
    /// Returns a `DomainError` if building the round-up transaction fails.
    pub fn process(&self, transaction: &Transaction) -> Result<Option<Transaction>, DomainError> {
        // Only trigger if the source account is credited (money leaving the account)
        let is_source_spending = transaction
            .postings()
            .iter()
            .any(|p| p.account() == &self.source_account && p.amount() < 0);

        if !is_source_spending {
            return Ok(None);
        }

        let mut spare_change = 0_i64;

        // Identify debits (expenses) that are not perfectly round dollars
        for posting in transaction.postings() {
            if posting.amount() > 0 {
                let remainder = posting.amount() % 100;
                if remainder > 0 {
                    spare_change = spare_change.saturating_add(100 - remainder);
                }
            }
        }

        if spare_change == 0 {
            return Ok(None);
        }

        let description = format!("Round-up: {}", transaction.description());

        let round_up_tx = TransactionBuilder::new(&description)
            .posting(Posting::credit(self.source_account.clone(), spare_change)?)
            .posting(Posting::debit(self.savings_account.clone(), spare_change)?)
            .build()?;

        Ok(Some(round_up_tx))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perfectly_round_amounts() {
        let checking = AccountId::new("assets:checking").unwrap();
        let savings = AccountId::new("assets:savings").unwrap();
        let food = AccountId::new("expenses:food").unwrap();

        let simulator = RoundUpSimulator::new(checking.clone(), savings);

        // $10.00 perfectly round
        let tx = TransactionBuilder::new("Groceries")
            .posting(Posting::debit(food, 1000).unwrap())
            .posting(Posting::credit(checking, 1000).unwrap())
            .build()
            .unwrap();

        let result = simulator.process(&tx).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_fractional_amounts_require_rounding_up() {
        let checking = AccountId::new("assets:checking").unwrap();
        let savings = AccountId::new("assets:savings").unwrap();
        let food = AccountId::new("expenses:food").unwrap();

        let simulator = RoundUpSimulator::new(checking.clone(), savings.clone());

        // $10.50 fractional
        let tx = TransactionBuilder::new("Groceries")
            .posting(Posting::debit(food, 1050).unwrap())
            .posting(Posting::credit(checking.clone(), 1050).unwrap())
            .build()
            .unwrap();

        let result = simulator.process(&tx).unwrap();
        assert!(result.is_some());

        let round_up_tx = result.unwrap();
        assert_eq!(round_up_tx.description(), "Round-up: Groceries");
        assert_eq!(round_up_tx.postings().len(), 2);

        // Spare change should be $0.50 (50 cents)
        let debit = round_up_tx
            .postings()
            .iter()
            .find(|p| p.amount() > 0)
            .unwrap();
        assert_eq!(debit.account(), &savings);
        assert_eq!(debit.amount(), 50);

        let credit = round_up_tx
            .postings()
            .iter()
            .find(|p| p.amount() < 0)
            .unwrap();
        assert_eq!(credit.account(), &checking);
        assert_eq!(credit.amount(), -50);
    }

    #[test]
    fn test_transaction_should_not_be_rounded_up_if_not_source_account() {
        let checking = AccountId::new("assets:checking").unwrap();
        let savings = AccountId::new("assets:savings").unwrap();
        let credit_card = AccountId::new("liabilities:credit_card").unwrap();
        let food = AccountId::new("expenses:food").unwrap();

        let simulator = RoundUpSimulator::new(checking, savings);

        // Spent $10.50 but from credit card, not checking
        let tx = TransactionBuilder::new("Groceries on CC")
            .posting(Posting::debit(food, 1050).unwrap())
            .posting(Posting::credit(credit_card, 1050).unwrap())
            .build()
            .unwrap();

        let result = simulator.process(&tx).unwrap();
        assert!(result.is_none());
    }
}
