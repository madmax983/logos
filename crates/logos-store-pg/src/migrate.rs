#![allow(clippy::missing_errors_doc)]
use diesel::migration::Migration;
use diesel::pg::PgConnection;
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};

use crate::error::PgStoreError;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub fn run_pending_migrations(conn: &mut PgConnection) -> Result<Vec<String>, PgStoreError> {
    let applied =
        conn.run_pending_migrations(MIGRATIONS)
            .map_err(|err| PgStoreError::Migration {
                message: err.to_string(),
            })?;

    Ok(applied
        .iter()
        .map(|migration| format!("{migration}"))
        .collect())
}

pub fn pending_migration_names(conn: &mut PgConnection) -> Result<Vec<String>, PgStoreError> {
    let pending = conn
        .pending_migrations(MIGRATIONS)
        .map_err(|err| PgStoreError::Migration {
            message: err.to_string(),
        })?;

    Ok(pending
        .iter()
        .map(|migration| format!("{}", migration.name().version()))
        .collect())
}
