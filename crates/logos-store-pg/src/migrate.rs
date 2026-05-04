//! Database Migration Management
use diesel::migration::Migration;
use diesel::pg::PgConnection;
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};

use logos_store::StoreError;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

/// Executes all pending database migrations against the provided connection.
///
/// This function applies the embedded migrations defined in `migrations/` that
/// have not yet been executed on the database. It returns a list of the migration
/// versions that were successfully applied.
///
/// ## Examples
///
/// ```no_run
/// use diesel::pg::PgConnection;
/// use diesel::Connection;
/// use logos_store_pg::run_pending_migrations;
///
/// let mut conn = PgConnection::establish("postgres://user:pass@localhost/db").unwrap();
/// let applied = run_pending_migrations(&mut conn).unwrap();
/// println!("Applied {} migrations", applied.len());
/// ```
///
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

/// Fetches the names of migrations that have not yet been applied to the database.
///
/// This is useful for checking the status of the database schema without actually
/// executing any changes. It returns a list of the versions (e.g. `20240101000000`)
/// of the pending migrations.
///
/// ## Examples
///
/// ```no_run
/// use diesel::pg::PgConnection;
/// use diesel::Connection;
/// use logos_store_pg::pending_migration_names;
///
/// let mut conn = PgConnection::establish("postgres://user:pass@localhost/db").unwrap();
/// let pending = pending_migration_names(&mut conn).unwrap();
/// if !pending.is_empty() {
///     println!("Database needs to be migrated!");
/// }
/// ```
///
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
