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
    /// Creates a normalized transaction id.
    ///
    /// # Errors
    ///
    /// Returns an error when `value` is empty after trimming.
    pub fn new(value: &str) -> Result<Self, DomainError> {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(DomainError::EmptyTransactionId);
        }

        Ok(Self(trimmed.to_owned()))
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
    /// When the new transaction id is already known, prefer
    /// [`Self::new_for_candidate`] to enforce the non-self-superseding invariant
    /// at construction time.
    ///
    /// ## Examples
    ///
    /// ```
    /// use logos_core::domain::correction::{Correction, TransactionId};
    ///
    /// let old_tx = TransactionId::new("tx-123")?;
    /// let correction = Correction::new(old_tx, "Fixed wrong account")?;
    /// assert_eq!(correction.reason(), "Fixed wrong account");
    /// # Ok::<(), logos_core::DomainError>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error when the provided `reason` is empty after trimming.
    ///
    /// ```
    /// use logos_core::domain::correction::{Correction, TransactionId};
    ///
    /// let old_tx = TransactionId::new("tx-123").expect("valid id");
    /// assert!(Correction::new(old_tx, "   ").is_err());
    /// ```
    pub fn new(supersedes_id: TransactionId, reason: &str) -> Result<Self, DomainError> {
        let trimmed_reason = reason.trim();
        if trimmed_reason.is_empty() {
            return Err(DomainError::EmptyCorrectionReason);
        }

        Ok(Self {
            supersedes_id,
            reason: trimmed_reason.to_owned(),
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

    /// Creates a correction and validates it against the new transaction id.
    ///
    /// This helper makes the "not self-superseding" invariant explicit at
    /// construction time whenever the candidate id is available.
    ///
    /// # Errors
    ///
    /// Returns an error when `reason` is empty after trimming, or when
    /// `candidate_id` equals `supersedes_id`.
    pub fn new_for_candidate(
        supersedes_id: TransactionId,
        candidate_id: &TransactionId,
        reason: &str,
    ) -> Result<Self, DomainError> {
        let correction = Self::new(supersedes_id, reason)?;
        correction.validate_not_self(candidate_id)?;
        Ok(correction)
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
    /// let old_tx = TransactionId::new("tx-123")?;
    /// let new_tx = TransactionId::new("tx-456")?;
    /// let correction = Correction::new(old_tx.clone(), "Typo")?;
    ///
    /// // Valid: new_tx != old_tx
    /// assert!(correction.validate_not_self(&new_tx).is_ok());
    ///
    /// // Invalid: A transaction cannot supersede its own ID.
    /// assert!(correction.validate_not_self(&old_tx).is_err());
    /// # Ok::<(), logos_core::DomainError>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error when `candidate_id` equals this correction's `supersedes_id`.
    pub fn validate_not_self(&self, candidate_id: &TransactionId) -> Result<(), DomainError> {
        if &self.supersedes_id == candidate_id {
            return Err(DomainError::CorrectionCannotSupersedeSelf);
        }

        Ok(())
    }
}
