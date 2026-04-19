use core::fmt;

use logos_core::{DomainError, TransactionId};

/// The definitive error type for all persistence operations.
///
/// When the `logos-store` boundary is crossed, things can fail. The database might go down,
/// a migration might panic, or you might try to reference a transaction that doesn't exist.
/// This enum provides clear recovery paths.
///
/// ## Examples
///
/// ```
/// use logos_store::StoreError;
/// use logos_core::TransactionId;
///
/// let err = StoreError::UnknownTransaction {
///     transaction_id: TransactionId::new("tx-123").unwrap(),
/// };
///
/// match err {
///     StoreError::UnknownTransaction { transaction_id } => {
///         println!("Could not find: {}", transaction_id.as_str());
///     }
///     _ => panic!("Expected UnknownTransaction"),
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoreError {
    /// A domain invariant was violated before storage even occurred.
    /// E.g., You tried to save a transaction that didn't balance.
    Domain(DomainError),
    /// You attempted to correct or lookup a transaction ID that the store has no record of.
    UnknownTransaction {
        /// The transaction id that was not found.
        transaction_id: TransactionId,
    },
    /// You attempted to lookup an analytics artifact that the store has no record of.
    UnknownArtifact {
        /// The artifact id that was not found.
        artifact_id: String,
    },
    /// Data failed to load from the underlying storage mechanism.
    LoadFailed {
        /// The specific error message.
        message: String,
    },
    /// Data failed to write to the underlying storage mechanism.
    PersistFailed {
        /// The specific error message.
        message: String,
    },
    /// Could not establish a connection to the database.
    ConnectionFailed {
        /// The specific error message.
        message: String,
    },
    /// Schema migrations failed to run.
    MigrationFailed {
        /// The specific error message.
        message: String,
    },
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
                write!(f, "{message}")
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
