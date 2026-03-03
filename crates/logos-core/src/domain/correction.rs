//! Immutability and corrections.
//!
//! In a strict double-entry ledger, transactions are generally append-only and
//! immutable. When a mistake is made, it is corrected by appending a new
//! transaction that explicitly "supersedes" the previous one, rather than
//! mutating the historical record in place.

use crate::error::DomainError;

/// A unique identifier for a recorded transaction.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TransactionId(String);

impl TransactionId {
    #[must_use]
    pub fn new(value: &str) -> Self {
        Self(value.to_owned())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A link from a new transaction to the previous transaction it corrects.
///
/// This structure enforces that a reason is provided for the change, leaving
/// an audit trail of modifications.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Correction {
    supersedes_id: TransactionId,
    reason: String,
}

impl Correction {
    /// Creates a correction pointing to an existing transaction id.
    ///
    /// A correction records the `TransactionId` that is being superseded,
    /// along with a mandatory reason explaining *why* the correction was made.
    ///
    /// ## Examples
    ///
    /// ```
    /// use logos_core::domain::correction::{Correction, TransactionId};
    ///
    /// let old_tx = TransactionId::new("tx-123");
    /// let correction = Correction::new(old_tx, "Fixed wrong account").unwrap();
    /// assert_eq!(correction.reason(), "Fixed wrong account");
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error when the provided `reason` is empty after trimming.
    ///
    /// ```
    /// use logos_core::domain::correction::{Correction, TransactionId};
    ///
    /// let old_tx = TransactionId::new("tx-123");
    /// assert!(Correction::new(old_tx, "   ").is_err());
    /// ```
    pub fn new(supersedes_id: TransactionId, reason: &str) -> Result<Self, DomainError> {
        if reason.trim().is_empty() {
            return Err(DomainError::EmptyCorrectionReason);
        }

        Ok(Self {
            supersedes_id,
            reason: reason.to_owned(),
        })
    }

    #[must_use]
    pub const fn supersedes_id(&self) -> &TransactionId {
        &self.supersedes_id
    }

    #[must_use]
    pub fn reason(&self) -> &str {
        &self.reason
    }

    /// Validates that a correction does not point to the new transaction's own ID.
    ///
    /// A transaction cannot supersede itself.
    ///
    /// ## Examples
    ///
    /// ```
    /// use logos_core::domain::correction::{Correction, TransactionId};
    ///
    /// let old_tx = TransactionId::new("tx-123");
    /// let new_tx = TransactionId::new("tx-456");
    /// let correction = Correction::new(old_tx.clone(), "Typo").unwrap();
    ///
    /// // Valid: new_tx != old_tx
    /// assert!(correction.clone().validate_not_self(&new_tx).is_ok());
    ///
    /// // Invalid: A transaction cannot supersede its own ID.
    /// assert!(correction.validate_not_self(&old_tx).is_err());
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error when `candidate_id` equals this correction's `supersedes_id`.
    pub fn validate_not_self(self, candidate_id: &TransactionId) -> Result<Self, DomainError> {
        if &self.supersedes_id == candidate_id {
            return Err(DomainError::CorrectionCannotSupersedeSelf);
        }

        Ok(self)
    }
}
