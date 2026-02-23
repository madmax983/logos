use crate::args::CliError;

/// Handles `ledger txn add`.
///
/// # Errors
///
/// Returns an error when the provided description is empty.
pub fn add(description: &str) -> Result<(), CliError> {
    if description.trim().is_empty() {
        return Err(CliError::MissingTxnDescription);
    }

    println!("txn.add accepted");
    Ok(())
}
