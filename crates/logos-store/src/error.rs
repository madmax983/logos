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

#[cfg(test)]
mod tests {
    use super::*;
    use logos_core::DomainError;

    #[test]
    fn test_store_error_display() {
        let domain_err = StoreError::Domain(DomainError::UnbalancedTransaction { total: 100 });
        assert_eq!(
            domain_err.to_string(),
            "transaction must be balanced to zero, but total was 100"
        );

        let unknown_tx_err = StoreError::UnknownTransaction {
            transaction_id: TransactionId::new("tx-123").unwrap(),
        };
        assert_eq!(
            unknown_tx_err.to_string(),
            "cannot apply correction: unknown transaction 'tx-123'"
        );

        let unknown_artifact_err = StoreError::UnknownArtifact {
            artifact_id: "art-456".to_string(),
        };
        assert_eq!(
            unknown_artifact_err.to_string(),
            "cannot link analytics artifact: unknown artifact 'art-456'"
        );

        let load_err = StoreError::LoadFailed {
            message: "db read timeout".to_string(),
        };
        assert_eq!(
            load_err.to_string(),
            "failed to load store: db read timeout"
        );

        let persist_err = StoreError::PersistFailed {
            message: "disk full".to_string(),
        };
        assert_eq!(
            persist_err.to_string(),
            "failed to persist store: disk full"
        );

        let conn_err = StoreError::ConnectionFailed {
            message: "connection refused".to_string(),
        };
        assert_eq!(conn_err.to_string(), "connection refused");

        let migrate_err = StoreError::MigrationFailed {
            message: "syntax error".to_string(),
        };
        assert_eq!(
            migrate_err.to_string(),
            "store migration failed: syntax error"
        );
    }

    #[test]
    fn test_store_error_from_domain_error() {
        let domain_err = DomainError::UnbalancedTransaction { total: 100 };
        let store_err: StoreError = domain_err.clone().into();
        match store_err {
            StoreError::Domain(e) => assert_eq!(e, domain_err),
            _ => panic!("Expected StoreError::Domain"),
        }
    }
}
