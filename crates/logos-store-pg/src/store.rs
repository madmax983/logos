use diesel::Connection;
use diesel::pg::PgConnection;

use crate::error::PgStoreError;
use crate::migrate::{pending_migration_names, run_pending_migrations};

pub struct PostgresStore {
    connection: PgConnection,
}

impl PostgresStore {
    pub fn connect(database_url: &str) -> Result<Self, PgStoreError> {
        if database_url.trim().is_empty() {
            return Err(PgStoreError::MissingDatabaseUrl);
        }

        let connection =
            PgConnection::establish(database_url).map_err(|err| PgStoreError::Connection {
                message: err.to_string(),
            })?;

        Ok(Self { connection })
    }

    #[must_use]
    pub fn connection_mut(&mut self) -> &mut PgConnection {
        &mut self.connection
    }

    pub fn pending_migrations(&mut self) -> Result<Vec<String>, PgStoreError> {
        pending_migration_names(&mut self.connection)
    }

    pub fn run_migrations(&mut self) -> Result<Vec<String>, PgStoreError> {
        run_pending_migrations(&mut self.connection)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connect_rejects_empty_database_url() {
        let err = PostgresStore::connect("   ")
            .err()
            .expect("empty DATABASE_URL must fail");
        assert!(matches!(err, PgStoreError::MissingDatabaseUrl));
    }
}
