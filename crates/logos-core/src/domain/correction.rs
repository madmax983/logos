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
    /// ## Examples
    ///
    /// ```
    /// use logos_core::domain::correction::TransactionId;
    ///
    /// let id = TransactionId::new(" tx-123 ").unwrap();
    /// assert_eq!(id.as_str(), "tx-123");
    /// ```
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

    /// Retrieves the string representation of the transaction id.
    ///
    /// ## Examples
    ///
    /// ```
    /// use logos_core::domain::correction::TransactionId;
    ///
    /// let id = TransactionId::new("tx-123").unwrap();
    /// assert_eq!(id.as_str(), "tx-123");
    /// ```
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

    /// Identifies the historical transaction that is being rewritten.
    ///
    /// The ledger is strictly append-only. To fix a mistake, a new transaction must
    /// be appended that explicitly points to the flawed entry it is replacing, ensuring
    /// a complete audit trail.
    ///
    /// ## Examples
    ///
    /// ```
    /// use logos_core::domain::correction::{Correction, TransactionId};
    ///
    /// let bad_tx = TransactionId::new("tx-err-1").unwrap();
    /// let correction = Correction::new(bad_tx.clone(), "Incorrect amount entered").unwrap();
    /// assert_eq!(correction.supersedes_id(), &bad_tx);
    /// ```
    #[must_use]
    pub const fn supersedes_id(&self) -> &TransactionId {
        &self.supersedes_id
    }

    /// Retrieves the mandatory reason for why the correction was made.
    ///
    /// ## Examples
    ///
    /// ```
    /// use logos_core::domain::correction::{Correction, TransactionId};
    ///
    /// let old_tx = TransactionId::new("tx-123").unwrap();
    /// let correction = Correction::new(old_tx, "Fixed wrong account").unwrap();
    /// assert_eq!(correction.reason(), "Fixed wrong account");
    /// ```
    #[must_use]
    pub fn reason(&self) -> &str {
        &self.reason
    }

    /// Creates a correction and validates it against the new transaction id.
    ///
    /// This helper makes the "not self-superseding" invariant explicit at
    /// construction time whenever the candidate id is available.
    ///
    /// ## Examples
    ///
    /// ```
    /// use logos_core::domain::correction::{Correction, TransactionId};
    ///
    /// let old_tx = TransactionId::new("tx-123").unwrap();
    /// let new_tx = TransactionId::new("tx-456").unwrap();
    /// let correction = Correction::new_for_candidate(old_tx, &new_tx, "Typo").unwrap();
    ///
    /// assert_eq!(correction.reason(), "Typo");
    /// ```
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_return_error_when_transaction_id_is_empty() {
        assert_eq!(TransactionId::new(""), Err(DomainError::EmptyTransactionId));
        assert_eq!(
            TransactionId::new("   "),
            Err(DomainError::EmptyTransactionId)
        );
    }

    #[test]
    fn should_return_error_when_correction_reason_is_empty() {
        let old_tx = TransactionId::new("tx-123").expect("valid id");
        assert_eq!(
            Correction::new(old_tx.clone(), ""),
            Err(DomainError::EmptyCorrectionReason)
        );
        assert_eq!(
            Correction::new(old_tx, "   "),
            Err(DomainError::EmptyCorrectionReason)
        );
    }

    #[test]
    fn should_return_error_when_correction_supersedes_self_in_validate() {
        let old_tx = TransactionId::new("tx-123").expect("valid id");
        let correction = Correction::new(old_tx.clone(), "Typo").unwrap();

        assert_eq!(
            correction.validate_not_self(&old_tx),
            Err(DomainError::CorrectionCannotSupersedeSelf)
        );
    }

    #[test]
    fn should_return_error_when_correction_supersedes_self_in_new_for_candidate() {
        let old_tx = TransactionId::new("tx-123").expect("valid id");
        assert_eq!(
            Correction::new_for_candidate(old_tx.clone(), &old_tx, "Typo"),
            Err(DomainError::CorrectionCannotSupersedeSelf)
        );
    }
}
