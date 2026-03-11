use crate::args::CliError;

/// Handles `ledger capture ingest`.
///
/// # Errors
///
/// Returns a placeholder runtime error until capture ingest is implemented.
pub fn ingest(_vault_path: &str, _inbox_subdir: Option<&str>) -> Result<(), CliError> {
    not_implemented("capture.ingest")
}

/// Handles `ledger capture list`.
///
/// # Errors
///
/// Returns a placeholder runtime error until capture listing is implemented.
pub fn list(_status: Option<&str>) -> Result<(), CliError> {
    not_implemented("capture.list")
}

/// Handles `ledger capture show`.
///
/// # Errors
///
/// Returns a placeholder runtime error until capture show is implemented.
pub fn show(_capture_id: &str) -> Result<(), CliError> {
    not_implemented("capture.show")
}

/// Handles `ledger capture promote`.
///
/// # Errors
///
/// Returns a placeholder runtime error until capture promotion is implemented.
pub fn promote(
    _capture_id: &str,
    _debit_account: Option<&str>,
    _credit_account: Option<&str>,
) -> Result<(), CliError> {
    not_implemented("capture.promote")
}

/// Handles `ledger capture reject`.
///
/// # Errors
///
/// Returns a placeholder runtime error until capture rejection is implemented.
pub fn reject(_capture_id: &str, _reason: &str) -> Result<(), CliError> {
    not_implemented("capture.reject")
}

fn not_implemented(command: &str) -> Result<(), CliError> {
    Err(CliError::CommandRuntimeFailed {
        command: command.to_owned(),
        message: "capture runtime is not implemented yet".to_owned(),
    })
}
