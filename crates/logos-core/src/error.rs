use core::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainError {
    EmptyTransactionDescription,
    EmptyCorrectionReason,
    CorrectionCannotSupersedeSelf,
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
