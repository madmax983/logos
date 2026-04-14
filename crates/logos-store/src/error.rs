use core::fmt;

use logos_core::{DomainError, TransactionId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoreError {
    Domain(DomainError),
    UnknownTransaction { transaction_id: TransactionId },
    UnknownArtifact { artifact_id: String },
    LoadFailed { message: String },
    PersistFailed { message: String },
    ConnectionFailed { message: String },
    MigrationFailed { message: String },
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Domain(err) => write!(f, "{err}"),
            Self::UnknownTransaction { transaction_id } => {
                write!(
                    f,
                    "Unknown transaction '{}'",
                    transaction_id.as_str()
                )
            }
            Self::UnknownArtifact { artifact_id } => {
                write!(
                    f,
                    "Unknown artifact '{artifact_id}'"
                )
            }
            Self::LoadFailed { message } => write!(f, "{message}"),
            Self::PersistFailed { message } => write!(f, "{message}"),
            Self::ConnectionFailed { message } => write!(f, "{message}"),
            Self::MigrationFailed { message } => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for StoreError {}

impl From<DomainError> for StoreError {
    fn from(value: DomainError) -> Self {
        Self::Domain(value)
    }
}
