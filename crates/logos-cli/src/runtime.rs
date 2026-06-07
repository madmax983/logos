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
    let database_url = env::var(DATABASE_URL_ENV).map_err(|_| CliError::CommandRuntimeFailed {
        command: "init".to_owned(),
        message: format!("{DATABASE_URL_ENV} is not set. Tip: Run `docker compose up -d db` and set DATABASE_URL (e.g. export DATABASE_URL=\"postgres://logos:logos@127.0.0.1:5432/logos\")"),
    })?;
    let mut store =
        PostgresStore::connect(&database_url).map_err(|err| CliError::CommandRuntimeFailed {
            command: "init".to_owned(),
            message: err.to_string(),
        })?;
    let pending = store
        .pending_migrations()
        .map_err(|err| CliError::CommandRuntimeFailed {
            command: "init".to_owned(),
            message: err.to_string(),
        })?;
    if !pending.is_empty() {
        let joined = pending.join(", ");
        return Err(CliError::CommandRuntimeFailed {
            command: "init".to_owned(),
            message: format!(
                "Pending database migrations detected ({joined}). Tip: Run `ledger db migrate` to apply them."
            ),
        });
    }
    let state_root = logos_runtime::default_state_root();
    Ok(AppRuntime::with_store(
        store,
        logos_runtime::default_artifacts_root(&state_root),
        Some(logos_runtime::default_fetch_config_path(&state_root)),
    ))
}
