use crate::args::CliError;
use logos_runtime::AppRuntime;
use logos_store_pg::PostgresStore;
use std::env;

const DATABASE_URL_ENV: &str = "DATABASE_URL";

///
/// # Errors
///
/// Returns an error when `DATABASE_URL` is missing, the database connection fails,
/// or applying migrations fails.
pub fn init_runtime() -> Result<AppRuntime<PostgresStore>, CliError> {
    println!("Loading history...");
    let database_url =
        env::var(DATABASE_URL_ENV).map_err(|_| CliError::CommandRuntimeFailed {
            command: "init".to_owned(),
            message: format!("{DATABASE_URL_ENV} is not set"),
        })?;
    let mut store = PostgresStore::connect(&database_url).map_err(|err| CliError::CommandRuntimeFailed {
        command: "init".to_owned(),
        message: err.to_string(),
    })?;
    let pending = store.pending_migrations().map_err(|err| CliError::CommandRuntimeFailed {
        command: "init".to_owned(),
        message: err.to_string(),
    })?;
    if !pending.is_empty() {
        let joined = pending.join(", ");
        return Err(CliError::CommandRuntimeFailed {
            command: "init".to_owned(),
            message: format!("pending database migrations detected ({joined}); run `ledger db migrate`"),
        });
    }
    println!("Database loaded.");
    let state_root = logos_runtime::runtime::default_state_root();
    Ok(AppRuntime::with_store(
        store,
        logos_runtime::runtime::default_artifacts_root(&state_root),
        Some(logos_runtime::runtime::default_fetch_config_path(&state_root)),
    ))
}
