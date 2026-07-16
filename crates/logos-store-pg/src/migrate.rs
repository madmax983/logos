//! Database Migration Management
use diesel::migration::Migration;
use diesel::pg::PgConnection;
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};

use logos_store::StoreError;

/// The embedded SQL migrations for the `PostgreSQL` database.
///
/// This constant bundles the raw SQL files found in the `migrations` directory
/// directly into the compiled binary.
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

/// # Errors
/// Returns `StoreError` on execution failure.
pub fn run_pending_migrations(conn: &mut PgConnection) -> Result<Vec<String>, StoreError> {
    let applied =
        conn.run_pending_migrations(MIGRATIONS)
            .map_err(|err| StoreError::MigrationFailed {
                message: err.to_string(),
            })?;

    Ok(applied
        .iter()
        .map(|migration| format!("{migration}"))
        .collect())
}

/// # Errors
/// Returns `StoreError` on fetch failure.
pub fn pending_migration_names(conn: &mut PgConnection) -> Result<Vec<String>, StoreError> {
    let pending =
        conn.pending_migrations(MIGRATIONS)
            .map_err(|err| StoreError::MigrationFailed {
                message: err.to_string(),
            })?;

    Ok(pending
        .iter()
        .map(|migration| format!("{}", migration.name().version()))
        .collect())
}
