use crate::error::DomainError;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Correction {
    supersedes_id: TransactionId,
    reason: String,
}

impl Correction {
    /// Creates an append-only correction pointing to an existing transaction id.
    ///
    /// # Errors
    ///
    /// Returns an error when the provided `reason` is empty after trimming.
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

    /// Rejects corrections that supersede the same transaction id supplied as `candidate_id`.
    ///
    /// # Errors
    ///
    /// Returns an error when `candidate_id` equals this correction `supersedes_id`.
    pub fn validate_not_self(self, candidate_id: &TransactionId) -> Result<Self, DomainError> {
        if &self.supersedes_id == candidate_id {
            return Err(DomainError::CorrectionCannotSupersedeSelf);
        }

        Ok(self)
    }
}
