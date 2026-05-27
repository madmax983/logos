use crate::args::CliError;
use crossterm::style::Stylize;
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
        println!("{} Database is up to date.", "✔".green());
        return Ok(());
    }

    let mut table = comfy_table::Table::new();
    table.load_preset(comfy_table::presets::UTF8_FULL_CONDENSED);
    table.set_header(vec![
        comfy_table::Cell::new("Status")
            .add_attribute(comfy_table::Attribute::Bold)
            .fg(comfy_table::Color::Green),
        comfy_table::Cell::new("Applied").add_attribute(comfy_table::Attribute::Bold),
        comfy_table::Cell::new("Migrations").add_attribute(comfy_table::Attribute::Bold),
    ]);

    let joined = applied.join(",");
    table.add_row(vec![
        comfy_table::Cell::new("✔ Success").fg(comfy_table::Color::Green),
        comfy_table::Cell::new(applied.len().to_string()),
        comfy_table::Cell::new(joined),
    ]);

    println!("{table}");
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
        println!(
            "{} Database is up to date. No pending migrations.",
            "✔".green()
        );
        return Ok(());
    }

    let mut table = comfy_table::Table::new();
    table.load_preset(comfy_table::presets::UTF8_FULL_CONDENSED);
    table.set_header(vec![
        comfy_table::Cell::new("Status")
            .add_attribute(comfy_table::Attribute::Bold)
            .fg(comfy_table::Color::Blue),
        comfy_table::Cell::new("Pending").add_attribute(comfy_table::Attribute::Bold),
        comfy_table::Cell::new("Migrations").add_attribute(comfy_table::Attribute::Bold),
    ]);

    let joined = pending.join(",");
    table.add_row(vec![
        comfy_table::Cell::new("ℹ Info").fg(comfy_table::Color::Blue),
        comfy_table::Cell::new(pending.len().to_string()),
        comfy_table::Cell::new(joined),
    ]);

    println!("{table}");
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
        message: err.to_string(),
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
        assert_eq!(err.to_string(), "DATABASE_URL is not set");
    }
}
