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
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum StoreError {
    /// A domain invariant was violated before storage even occurred.
    /// E.g., You tried to save a transaction that didn't balance.
    #[error(transparent)]
    Domain(#[from] DomainError),
    /// You attempted to correct or lookup a transaction ID that the store has no record of.
    #[error("cannot apply correction: unknown transaction '{0}'", transaction_id.as_str())]
    UnknownTransaction {
        /// The transaction id that was not found.
        transaction_id: TransactionId,
    },
    /// You attempted to lookup an analytics artifact that the store has no record of.
    #[error("cannot link analytics artifact: unknown artifact '{artifact_id}'")]
    UnknownArtifact {
        /// The artifact id that was not found.
        artifact_id: String,
    },
    /// Data failed to load from the underlying storage mechanism.
    #[error("failed to load store: {message}")]
    LoadFailed {
        /// The specific error message.
        message: String,
    },
    /// Data failed to write to the underlying storage mechanism.
    #[error("failed to persist store: {message}")]
    PersistFailed {
        /// The specific error message.
        message: String,
    },
    /// Could not establish a connection to the database.
    #[error("{message}")]
    ConnectionFailed {
        /// The specific error message.
        message: String,
    },
    /// Schema migrations failed to run.
    #[error("store migration failed: {message}")]
    MigrationFailed {
        /// The specific error message.
        message: String,
    },
}
