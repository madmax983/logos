use core::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainError {
    EmptyTransactionDescription,
    EmptyCorrectionReason,
    CorrectionCannotSupersedeSelf,
    InvalidAllocationTotal { total: u16 },
    UnbalancedTransaction { total: i64 },
}

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyTransactionDescription => {
                write!(f, "transaction description cannot be empty")
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
            Self::UnbalancedTransaction { total } => {
                write!(
                    f,
                    "transaction must be balanced to zero, but total was {total}"
                )
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
    fn should_display_empty_correction_reason() {
        assert_eq!(
            DomainError::EmptyCorrectionReason.to_string(),
            "correction reason cannot be empty"
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
}
