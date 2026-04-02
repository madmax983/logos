use crate::args::CliError;
use logos_store_pg::PostgresStore;

const DATABASE_URL_ENV: &str = "DATABASE_URL";

/// Handles `ledger db migrate`.
///
/// # Errors
///
/// Returns an error when `DATABASE_URL` is missing, the database connection fails,
/// or applying migrations fails.
pub fn migrate() -> Result<(), CliError> {
    let mut store = connect_store("db.migrate")?;
    let applied = store
        .run_migrations()
        .map_err(|err| CliError::CommandRuntimeFailed {
            command: "db.migrate".to_owned(),
            message: format!("migration execution failed: {err}"),
        })?;

    if applied.is_empty() {
        println!("db.migrate applied=0 status=up_to_date");
        return Ok(());
    }

    let joined = applied.join(",");
    println!("db.migrate applied={} names={joined}", applied.len());
    Ok(())
}

/// Handles `ledger db status`.
///
/// # Errors
///
/// Returns an error when `DATABASE_URL` is missing, the database connection fails,
/// or pending migrations cannot be inspected.
pub fn status() -> Result<(), CliError> {
    let mut store = connect_store("db.status")?;
    let pending = store
        .pending_migrations()
        .map_err(|err| CliError::CommandRuntimeFailed {
            command: "db.status".to_owned(),
            message: format!("failed to inspect pending migrations: {err}"),
        })?;

    if pending.is_empty() {
        println!("db.status pending=0 status=up_to_date");
        return Ok(());
    }

    let joined = pending.join(",");
    println!("db.status pending={} names={joined}", pending.len());
    Ok(())
}

fn connect_store(command: &str) -> Result<PostgresStore, CliError> {
    let database_url =
        std::env::var(DATABASE_URL_ENV).map_err(|_| CliError::CommandRuntimeFailed {
            command: command.to_owned(),
            message: format!("The {} environment variable is not set.\nPlease provide a valid Postgres connection string (e.g. export {}=\"postgres://user:pass@localhost:5432/logos\").", DATABASE_URL_ENV, DATABASE_URL_ENV),
        })?;
    PostgresStore::connect(&database_url).map_err(|err| CliError::CommandRuntimeFailed {
        command: command.to_owned(),
        message: format!("Failed to connect to the database.\nEnsure Postgres is running and the connection string is correct.\nUnderlying error: {err}"),
    })
}

#[cfg(test)]
mod tests {
    use super::connect_store;

    #[test]
    fn connect_store_requires_database_url() {
        let previous = std::env::var_os("DATABASE_URL");
        unsafe {
            std::env::remove_var("DATABASE_URL");
        }

        let err = match connect_store("db.status") {
            Ok(_) => panic!("missing database url must fail"),
            Err(e) => e,
        };
        assert_eq!(
            err.to_string(),
            "Command 'db.status' failed:\n  The DATABASE_URL environment variable is not set.\nPlease provide a valid Postgres connection string (e.g. export DATABASE_URL=\"postgres://user:pass@localhost:5432/logos\")."
        );

        if let Some(value) = previous {
            unsafe {
                std::env::set_var("DATABASE_URL", value);
            }
        }
    }
}
