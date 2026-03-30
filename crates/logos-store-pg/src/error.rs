use thiserror::Error;

#[derive(Debug, Error)]
pub enum PgStoreError {
    #[error("DATABASE_URL must not be empty")]
    MissingDatabaseUrl,

    #[error("failed to connect to postgres: {message}")]
    Connection { message: String },

    #[error("migration error: {message}")]
    Migration { message: String },
}
