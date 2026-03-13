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
///
/// // Create a debit posting for $10.00 (1000 cents).
/// let d = Posting::debit("assets:checking", 1000).unwrap();
/// assert_eq!(d.amount(), 1000);
///
/// // Create a credit posting for $10.00 (represented as -1000 cents internally).
/// let c = Posting::credit("income:salary", 1000).unwrap();
/// assert_eq!(c.amount(), -1000);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Posting {
    account: String,
    amount: i64,
}

impl Posting {
    /// Creates a debit posting. The amount should be passed as a positive number.
    ///
    /// # Errors
    ///
    /// Returns an error if the amount is negative.
    pub fn debit(account: &str, amount: i64) -> Result<Self, DomainError> {
        if amount < 0 {
            return Err(DomainError::AmountOverflow);
        }
        Ok(Self {
            account: account.to_owned(),
            amount,
        })
    }

    /// Creates a credit posting. The `amount` parameter is passed as positive,
    /// but will be negated internally to maintain the credit sign convention.
    ///
    /// # Errors
    ///
    /// Returns an error if the amount is negative or overflows.
    pub fn credit(account: &str, amount: i64) -> Result<Self, DomainError> {
        let negated = amount.checked_neg().ok_or(DomainError::AmountOverflow)?;
        if amount < 0 {
            return Err(DomainError::AmountOverflow);
        }
        Ok(Self {
            account: account.to_owned(),
            amount: negated,
        })
    }

    #[must_use]
    pub fn account(&self) -> &str {
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
    #[must_use]
    pub fn description(&self) -> &str {
        &self.description
    }

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
///
/// // A successful balanced transaction:
/// let txn = TransactionBuilder::new("Buy groceries")
///     .posting(Posting::debit("expenses:food", 5000).unwrap())
///     .posting(Posting::credit("assets:checking", 5000).unwrap())
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
///
/// let result = TransactionBuilder::new("Oops")
///     .posting(Posting::debit("expenses:food", 5000).unwrap())
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
    #[must_use]
    pub fn new(description: &str) -> Self {
        Self {
            description: description.to_owned(),
            postings: Vec::new(),
        }
    }

    /// Adds a [`Posting`] to the transaction.
    #[must_use]
    pub fn posting(mut self, posting: Posting) -> Self {
        self.postings.push(posting);
        self
    }

    /// Builds a transaction after enforcing write-time balancing.
    ///
    /// # Errors
    ///
    /// Returns an error when the description is empty or postings do not sum to zero.
    pub fn build(self) -> Result<Transaction, DomainError> {
        if self.description.trim().is_empty() {
            return Err(DomainError::EmptyTransactionDescription);
        }

        let mut total: i64 = 0;
        for p in &self.postings {
            total = total
                .checked_add(p.amount())
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
