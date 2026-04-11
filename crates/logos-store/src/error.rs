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
                    "cannot apply correction: unknown transaction '{}'",
                    transaction_id.as_str()
                )
            }
            Self::UnknownArtifact { artifact_id } => {
                write!(
                    f,
                    "cannot link analytics artifact: unknown artifact '{artifact_id}'"
                )
            }
            Self::LoadFailed { message } => write!(f, "failed to load store: {message}"),
            Self::PersistFailed { message } => write!(f, "failed to persist store: {message}"),
            Self::ConnectionFailed { message } => {
                write!(f, "failed to connect to store: {message}")
            }
            Self::MigrationFailed { message } => write!(f, "store migration failed: {message}"),
        }
    }
}

impl std::error::Error for StoreError {}

impl From<DomainError> for StoreError {
    fn from(value: DomainError) -> Self {
        Self::Domain(value)
    }
}
