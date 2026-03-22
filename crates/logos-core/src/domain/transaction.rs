//! Core double-entry accounting structures for `logos`.
//!
//! The `transaction` module contains the fundamental structures for recording
//! financial events. In `logos`, all amounts are represented in **cents** to
//! avoid floating-point precision issues.
//!
//! **Critical Concept:**
//! - **Debits** are always **positive** values.
//! - **Credits** are always **negative** values.
//!
//! A [`Transaction`] must always balance to zero before it can be created. This
//! invariant is enforced via the [`TransactionBuilder`].

use crate::domain::account::AccountId;
use crate::error::DomainError;

/// A single line item within a [`Transaction`].
///
/// A posting affects a single account and has a monetary amount. In double-entry
/// accounting, postings can be debits or credits. This struct ensures consistency
/// by storing amounts with a strict sign convention.
///
/// ## Sign Convention
/// * **Debits** are strictly **positive** values (e.g. `1000` = +$10.00).
/// * **Credits** are strictly **negative** values (e.g. `-1000` = -$10.00).
///
/// ## Examples
///
/// ```
/// use logos_core::domain::transaction::Posting;
/// use logos_core::domain::account::AccountId;
///
/// // Create a debit posting for $10.00 (1000 cents).
/// let d = Posting::debit(AccountId::new("assets:checking").unwrap(), 1000).expect("debit should succeed");
/// assert_eq!(d.amount(), 1000);
///
/// // Create a credit posting for $10.00 (represented as -1000 cents internally).
/// let c = Posting::credit(AccountId::new("income:salary").unwrap(), 1000).expect("credit should succeed");
/// assert_eq!(c.amount(), -1000);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Posting {
    account: AccountId,
    amount: i64,
}

impl Posting {
    /// Creates a debit posting.
    ///
    /// ## Examples
    ///
    /// ```
    /// use logos_core::domain::transaction::Posting;
    /// use logos_core::AccountId;
    ///
    /// let posting = Posting::debit(AccountId::new("assets:checking").unwrap(), 1000).unwrap();
    /// assert_eq!(posting.amount(), 1000);
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error when `amount` is not strictly positive.
    pub fn debit(account: AccountId, amount: i64) -> Result<Self, DomainError> {
        if amount <= 0 {
            return Err(DomainError::InvalidDebitAmount { amount });
        }

        Ok(Self { account, amount })
    }

    /// Creates a credit posting, negating the provided amount.
    ///
    /// ## Examples
    ///
    /// ```
    /// use logos_core::domain::transaction::Posting;
    /// use logos_core::AccountId;
    ///
    /// let posting = Posting::credit(AccountId::new("income:salary").unwrap(), 1000).unwrap();
    /// assert_eq!(posting.amount(), -1000); // Credits are strictly negative
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error when `amount` is not strictly positive.
    /// Returns an error if negating `amount` causes an arithmetic overflow.
    pub fn credit(account: AccountId, amount: i64) -> Result<Self, DomainError> {
        if amount <= 0 {
            return Err(DomainError::InvalidCreditAmount { amount });
        }

        let amount = amount.checked_neg().ok_or(DomainError::AmountOverflow)?;
        Ok(Self { account, amount })
    }

    #[must_use]
    pub const fn account(&self) -> &AccountId {
        &self.account
    }

    #[must_use]
    pub const fn amount(&self) -> i64 {
        self.amount
    }
}

/// A balanced double-entry financial transaction.
///
/// Transactions cannot be instantiated directly; you must use a
/// [`TransactionBuilder`] to guarantee that the sum of all its [`Posting`]
/// amounts equals zero.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transaction {
    description: String,
    postings: Vec<Posting>,
}

impl Transaction {
    /// Retrieves the description of the transaction.
    ///
    /// ## Examples
    ///
    /// ```
    /// use logos_core::domain::transaction::{TransactionBuilder, Posting};
    /// use logos_core::AccountId;
    ///
    /// let txn = TransactionBuilder::new("Buy groceries")
    ///     .posting(Posting::debit(AccountId::new("expenses:food").unwrap(), 5000).unwrap())
    ///     .posting(Posting::credit(AccountId::new("assets:checking").unwrap(), 5000).unwrap())
    ///     .build()
    ///     .unwrap();
    ///
    /// assert_eq!(txn.description(), "Buy groceries");
    /// ```
    #[must_use]
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Retrieves the list of postings associated with the transaction.
    ///
    /// ## Examples
    ///
    /// ```
    /// use logos_core::domain::transaction::{TransactionBuilder, Posting};
    /// use logos_core::AccountId;
    ///
    /// let txn = TransactionBuilder::new("Buy groceries")
    ///     .posting(Posting::debit(AccountId::new("expenses:food").unwrap(), 5000).unwrap())
    ///     .posting(Posting::credit(AccountId::new("assets:checking").unwrap(), 5000).unwrap())
    ///     .build()
    ///     .unwrap();
    ///
    /// assert_eq!(txn.postings().len(), 2);
    /// ```
    #[must_use]
    pub fn postings(&self) -> &[Posting] {
        &self.postings
    }
}

/// A builder for creating valid, balanced [`Transaction`]s.
///
/// To prevent corrupted ledger states, all transactions must balance. The builder
/// accumulates postings and enforces that their total sum equals exactly `0` when
/// [`build`](Self::build) is called.
///
/// ## Examples
///
/// ```
/// use logos_core::domain::transaction::{TransactionBuilder, Posting};
/// use logos_core::AccountId;
///
/// // A successful balanced transaction:
/// let txn = TransactionBuilder::new("Buy groceries")
///     .posting(Posting::debit(AccountId::new("expenses:food").unwrap(), 5000).expect("debit should succeed"))
///     .posting(Posting::credit(AccountId::new("assets:checking").unwrap(), 5000).expect("credit should succeed"))
///     .build()
///     .expect("Transaction should balance");
///
/// assert_eq!(txn.postings().len(), 2);
/// ```
///
/// Unbalanced transactions will return a `DomainError`:
///
/// ```
/// use logos_core::domain::transaction::{TransactionBuilder, Posting};
/// use logos_core::AccountId;
///
/// let result = TransactionBuilder::new("Oops")
///     .posting(Posting::debit(AccountId::new("expenses:food").unwrap(), 5000).expect("debit should succeed"))
///     .build(); // Missing the credit!
///
/// assert!(result.is_err());
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TransactionBuilder {
    description: String,
    postings: Vec<Posting>,
}

impl TransactionBuilder {
    /// Initiates a new transaction builder with the given description.
    ///
    /// ## Examples
    ///
    /// ```
    /// use logos_core::domain::transaction::TransactionBuilder;
    ///
    /// let builder = TransactionBuilder::new("Buy groceries");
    /// ```
    #[must_use]
    pub fn new(description: &str) -> Self {
        Self {
            description: description.to_owned(),
            postings: Vec::new(),
        }
    }

    /// Adds a [`Posting`] to the transaction.
    ///
    /// ## Examples
    ///
    /// ```
    /// use logos_core::domain::transaction::{TransactionBuilder, Posting};
    /// use logos_core::AccountId;
    ///
    /// let builder = TransactionBuilder::new("Buy groceries")
    ///     .posting(Posting::debit(AccountId::new("expenses:food").unwrap(), 5000).unwrap());
    /// ```
    #[must_use]
    pub fn posting(mut self, posting: Posting) -> Self {
        self.postings.push(posting);
        self
    }

    /// Builds a transaction after enforcing write-time balancing.
    ///
    /// # Errors
    ///
    /// Returns an error when the description is empty, there are no postings,
    /// or postings do not sum to zero.
    ///
    /// ## Examples
    ///
    /// ```
    /// use logos_core::domain::transaction::{TransactionBuilder, Posting};
    /// use logos_core::AccountId;
    ///
    /// let txn = TransactionBuilder::new("Buy groceries")
    ///     .posting(Posting::debit(AccountId::new("expenses:food").unwrap(), 5000).unwrap())
    ///     .posting(Posting::credit(AccountId::new("assets:checking").unwrap(), 5000).unwrap())
    ///     .build();
    ///
    /// assert!(txn.is_ok());
    /// ```
    pub fn build(self) -> Result<Transaction, DomainError> {
        if self.description.trim().is_empty() {
            return Err(DomainError::EmptyTransactionDescription);
        }

        if self.postings.is_empty() {
            return Err(DomainError::EmptyTransactionPostings);
        }

        let mut total = 0_i64;
        for posting in &self.postings {
            total = total
                .checked_add(posting.amount())
                .ok_or(DomainError::AmountOverflow)?;
        }

        if total != 0 {
            return Err(DomainError::UnbalancedTransaction { total });
        }

        Ok(Transaction {
            description: self.description,
            postings: self.postings,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_return_error_when_posting_credit_negates_min_value() {
        let result = Posting::credit(AccountId::new("income:salary").unwrap(), i64::MIN);
        assert_eq!(
            result,
            Err(DomainError::InvalidCreditAmount { amount: i64::MIN })
        );
    }

    #[test]
    fn should_return_error_when_posting_debit_amount_is_non_positive() {
        let account = AccountId::new("assets:checking").expect("account");
        assert_eq!(
            Posting::debit(account.clone(), 0),
            Err(DomainError::InvalidDebitAmount { amount: 0 })
        );
        assert_eq!(
            Posting::debit(account, -1),
            Err(DomainError::InvalidDebitAmount { amount: -1 })
        );
    }

    #[test]
    fn should_return_error_when_posting_credit_amount_is_non_positive() {
        let account = AccountId::new("income:salary").expect("account");
        assert_eq!(
            Posting::credit(account.clone(), 0),
            Err(DomainError::InvalidCreditAmount { amount: 0 })
        );
        assert_eq!(
            Posting::credit(account, -1),
            Err(DomainError::InvalidCreditAmount { amount: -1 })
        );
    }

    #[test]
    fn should_return_error_when_transaction_has_no_postings() {
        let result = TransactionBuilder::new("empty").build();
        assert_eq!(result, Err(DomainError::EmptyTransactionPostings));
    }

    #[test]
    fn should_return_error_when_transaction_sum_overflows_positive() {
        let builder = TransactionBuilder::new("overflow")
            .posting(
                Posting::debit(AccountId::new("assets:checking").unwrap(), i64::MAX)
                    .expect("debit"),
            )
            .posting(Posting::debit(AccountId::new("assets:checking").unwrap(), 2).expect("debit"))
            .posting(
                Posting::credit(AccountId::new("income:salary").unwrap(), i64::MAX)
                    .expect("credit"),
            )
            .posting(Posting::credit(AccountId::new("income:salary").unwrap(), 2).expect("credit"));

        let result = builder.build();
        assert_eq!(result, Err(DomainError::AmountOverflow));
    }

    #[test]
    fn should_return_error_when_transaction_sum_overflows_negative() {
        let builder = TransactionBuilder::new("underflow")
            .posting(
                Posting::credit(AccountId::new("income:salary").unwrap(), i64::MAX)
                    .expect("credit"),
            )
            .posting(Posting::credit(AccountId::new("income:salary").unwrap(), 2).expect("credit"))
            .posting(
                Posting::debit(AccountId::new("assets:checking").unwrap(), i64::MAX)
                    .expect("debit"),
            )
            .posting(Posting::debit(AccountId::new("assets:checking").unwrap(), 2).expect("debit"));

        let result = builder.build();
        assert_eq!(result, Err(DomainError::AmountOverflow));
    }
}
