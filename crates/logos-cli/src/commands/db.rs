use crate::args::CliError;
use logos_store_pg::PostgresStore;
use crossterm::style::Stylize;

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
        println!("{} Database is up to date.", "✔".green());
        return Ok(());
    }

    let joined = applied.join(",");
    println!("{} Applied {} migration(s): {}", "✔".green(), applied.len(), joined.bold());
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
        println!("{} Database is up to date. No pending migrations.", "✔".green());
        return Ok(());
    }

    let joined = pending.join(",");
    println!("{} {} pending migration(s): {}", "ℹ".blue(), pending.len(), joined.bold());
    Ok(())
}

fn connect_store_with_env(
    command: &str,
    env_var: Option<String>,
) -> Result<PostgresStore, CliError> {
    let database_url = env_var.ok_or_else(|| CliError::CommandRuntimeFailed {
        command: command.to_owned(),
        message: "DATABASE_URL is not set".to_owned(),
    })?;
    PostgresStore::connect(&database_url).map_err(|err| CliError::CommandRuntimeFailed {
        command: command.to_owned(),
        message: format!("database connection failed: {err}"),
    })
}

fn connect_store(command: &str) -> Result<PostgresStore, CliError> {
    connect_store_with_env(command, std::env::var(DATABASE_URL_ENV).ok())
}

#[cfg(test)]
mod tests {
    use super::connect_store_with_env;

    #[test]
    fn connect_store_requires_database_url() {
        let Err(err) = connect_store_with_env("db.status", None) else {
            panic!("missing database url must fail");
        };
        assert_eq!(
            err.to_string(),
            "Command 'db.status' failed: DATABASE_URL is not set."
        );
    }
}
