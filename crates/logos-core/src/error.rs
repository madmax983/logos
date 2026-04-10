//! Error handling and recovery for the `logos` domain.
//!
//! This module defines the [`DomainError`] enumeration, which represents every
//! possible failure mode that can occur when constructing valid financial
//! primitives in the core library.
//!
//! Because `logos` is a strict, double-entry system, it rejects invalid states
//! at creation time. If a function returns a `DomainError`, it means you tried
//! to perform an operation that violates fundamental accounting or forecasting rules.

use core::fmt;

/// The central error type for all financial constraints in `logos`.
///
/// This enumeration captures all the ways you might accidentally construct
/// an invalid ledger state, such as an unbalanced transaction, an empty account name,
/// or a nonsensical risk haircut.
///
/// ## Examples
///
/// When an operation fails in the `domain`, it returns one of these variants.
/// You can match on the specific variant to provide helpful error messages
/// or recovery strategies to the user.
///
/// ```
/// use logos_core::{AccountId, DomainError, Posting, TransactionBuilder};
///
/// let account = AccountId::new("assets:cash").unwrap();
///
/// // Attempt to build an unbalanced transaction (missing the credit leg).
/// let result = TransactionBuilder::new("Bought a coffee")
///     .posting(Posting::debit(account, 5_00).unwrap())
///     .build();
///
/// match result {
///     Ok(_) => panic!("This should not happen!"),
///     Err(DomainError::UnbalancedTransaction { total }) => {
///         assert_eq!(total, 5_00);
///         println!("Transaction is out of balance by {} cents!", total);
///     }
///     Err(e) => panic!("Unexpected error: {:?}", e),
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainError {
    /// Returned when you try to create an `AccountId` from an empty string or just whitespace.
    EmptyAccountId,
    /// Returned when you try to create a `TransactionId` from an empty string or just whitespace.
    EmptyTransactionId,
    /// Returned when a `CategoryGroup` is given a name that trims down to nothing.
    EmptyCategoryGroupName,
    /// Returned when a `Category` is given a name that trims down to nothing.
    EmptyCategoryName,
    /// Returned when a transaction builder is given an empty description.
    /// Every transaction must explain *what* happened.
    EmptyTransactionDescription,
    /// Returned when you attempt to build a transaction without any postings.
    /// A double-entry transaction needs at least two legs!
    EmptyTransactionPostings,
    /// Returned when a correction is created without a reason. You must explain *why*
    /// you are rewriting history.
    EmptyCorrectionReason,
    /// Returned when a correction claims to supersede its own ID.
    /// You cannot write a transaction that replaces itself.
    CorrectionCannotSupersedeSelf,
    /// Returned when an RSU allocation policy does not equal exactly 100%.
    /// The `total` field tells you what sum you provided.
    InvalidAllocationTotal { total: u16 },
    /// Returned when a specific time horizon in a haircut table exceeds 100%.
    /// `tier` indicates the culprit (e.g., "short" or "medium"), and `percentage` is what you tried to set.
    InvalidHaircutPercentage { tier: &'static str, percentage: u8 },
    /// Returned when your risk haircut tiers are backwards.
    /// Risk should increase over time, so you must have `short <= medium <= long`.
    InvalidHaircutOrdering { short: u8, medium: u8, long: u8 },
    /// Returned when you pass a zero or negative amount to a debit. Debits must be strictly positive.
    InvalidDebitAmount { amount: i64 },
    /// Returned when you pass a zero or negative amount to a credit. Credits must be strictly positive
    /// before they are converted internally to negatives.
    InvalidCreditAmount { amount: i64 },
    /// The fundamental rule of accounting broken. A transaction's debits and credits
    /// must perfectly cancel each other out to `0`. The `total` tells you how far off balance you are.
    UnbalancedTransaction { total: i64 },
    /// Returned when an operation exceeds the bounds of a 64-bit signed integer.
    AmountOverflow,
}

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyAccountId => {
                write!(f, "account id cannot be empty")
            }
            Self::EmptyTransactionId => {
                write!(f, "transaction id cannot be empty")
            }
            Self::EmptyCategoryGroupName => {
                write!(f, "category group name cannot be empty")
            }
            Self::EmptyCategoryName => {
                write!(f, "category name cannot be empty")
            }
            Self::EmptyTransactionDescription => {
                write!(f, "transaction description cannot be empty")
            }
            Self::EmptyTransactionPostings => {
                write!(f, "transaction must contain at least one posting")
            }
            Self::EmptyCorrectionReason => {
                write!(f, "correction reason cannot be empty")
            }
            Self::CorrectionCannotSupersedeSelf => {
                write!(f, "correction cannot supersede itself")
            }
            Self::InvalidAllocationTotal { total } => {
                write!(f, "allocation percentages must sum to 100, got {total}")
            }
            Self::InvalidHaircutPercentage { tier, percentage } => {
                write!(
                    f,
                    "haircut percentage for {tier} tier must be in 0..=100, got {percentage}"
                )
            }
            Self::InvalidHaircutOrdering {
                short,
                medium,
                long,
            } => {
                write!(
                    f,
                    "haircut tiers must be non-decreasing by horizon (short <= medium <= long), got {short}, {medium}, {long}"
                )
            }
            Self::InvalidDebitAmount { amount } => {
                write!(f, "debit amount must be greater than zero, got {amount}")
            }
            Self::InvalidCreditAmount { amount } => {
                write!(f, "credit amount must be greater than zero, got {amount}")
            }
            Self::UnbalancedTransaction { total } => {
                write!(
                    f,
                    "transaction must be balanced to zero, but total was {total}"
                )
            }
            Self::AmountOverflow => {
                write!(f, "transaction amount calculation resulted in an overflow")
            }
        }
    }
}

impl std::error::Error for DomainError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_display_empty_transaction_description() {
        assert_eq!(
            DomainError::EmptyTransactionDescription.to_string(),
            "transaction description cannot be empty"
        );
    }

    #[test]
    fn should_display_empty_transaction_postings() {
        assert_eq!(
            DomainError::EmptyTransactionPostings.to_string(),
            "transaction must contain at least one posting"
        );
    }

    #[test]
    fn should_display_empty_account_id() {
        assert_eq!(
            DomainError::EmptyAccountId.to_string(),
            "account id cannot be empty"
        );
    }

    #[test]
    fn should_display_empty_correction_reason() {
        assert_eq!(
            DomainError::EmptyCorrectionReason.to_string(),
            "correction reason cannot be empty"
        );
    }

    #[test]
    fn should_display_empty_transaction_id() {
        assert_eq!(
            DomainError::EmptyTransactionId.to_string(),
            "transaction id cannot be empty"
        );
    }

    #[test]
    fn should_display_correction_cannot_supersede_self() {
        assert_eq!(
            DomainError::CorrectionCannotSupersedeSelf.to_string(),
            "correction cannot supersede itself"
        );
    }

    #[test]
    fn should_display_invalid_allocation_total() {
        assert_eq!(
            DomainError::InvalidAllocationTotal { total: 105 }.to_string(),
            "allocation percentages must sum to 100, got 105"
        );
    }

    #[test]
    fn should_display_unbalanced_transaction() {
        assert_eq!(
            DomainError::UnbalancedTransaction { total: -500 }.to_string(),
            "transaction must be balanced to zero, but total was -500"
        );
    }

    #[test]
    fn should_display_invalid_haircut_percentage() {
        assert_eq!(
            DomainError::InvalidHaircutPercentage {
                tier: "long",
                percentage: 120
            }
            .to_string(),
            "haircut percentage for long tier must be in 0..=100, got 120"
        );
    }

    #[test]
    fn should_display_invalid_haircut_ordering() {
        assert_eq!(
            DomainError::InvalidHaircutOrdering {
                short: 50,
                medium: 40,
                long: 60
            }
            .to_string(),
            "haircut tiers must be non-decreasing by horizon (short <= medium <= long), got 50, 40, 60"
        );
    }

    #[test]
    fn should_display_invalid_debit_amount() {
        assert_eq!(
            DomainError::InvalidDebitAmount { amount: 0 }.to_string(),
            "debit amount must be greater than zero, got 0"
        );
    }

    #[test]
    fn should_display_invalid_credit_amount() {
        assert_eq!(
            DomainError::InvalidCreditAmount { amount: -1 }.to_string(),
            "credit amount must be greater than zero, got -1"
        );
    }

    #[test]
    fn should_display_empty_category_group_name() {
        assert_eq!(
            DomainError::EmptyCategoryGroupName.to_string(),
            "category group name cannot be empty"
        );
    }

    #[test]
    fn should_display_empty_category_name() {
        assert_eq!(
            DomainError::EmptyCategoryName.to_string(),
            "category name cannot be empty"
        );
    }

    #[test]
    fn should_display_amount_overflow() {
        assert_eq!(
            DomainError::AmountOverflow.to_string(),
            "transaction amount calculation resulted in an overflow"
        );
    }
}
