use std::path::Path;

use crate::{
    args::CliError,
    runtime::{CaptureIngestSummary, CliRuntime},
};

/// Handles `ledger capture ingest`.
///
/// # Errors
///
/// Returns an error when runtime initialization or capture ingest fails.
pub fn ingest(vault_path: &str, inbox_subdir: Option<&str>) -> Result<(), CliError> {
    let mut runtime = CliRuntime::new().map_err(|err| CliError::CommandRuntimeFailed {
        command: "capture.ingest".to_owned(),
        message: format!("runtime initialization failed: {err}"),
    })?;
    let summary = runtime
        .ingest_capture_notes(Path::new(vault_path), inbox_subdir)
        .map_err(|err| CliError::CommandRuntimeFailed {
            command: "capture.ingest".to_owned(),
            message: err.to_string(),
        })?;
    println!(
        "{}",
        render_ingest_output(vault_path, inbox_subdir, summary)
    );
    Ok(())
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

fn render_ingest_output(
    vault_path: &str,
    inbox_subdir: Option<&str>,
    summary: CaptureIngestSummary,
) -> String {
    format!(
        "capture.ingest vault_path={} inbox_subdir={} ingested={} updated={} skipped={} conflict={} malformed={}",
        vault_path,
        inbox_subdir.unwrap_or("finance/inbox"),
        summary.ingested_count(),
        summary.updated_count(),
        summary.skipped_count(),
        summary.conflict_count(),
        summary.malformed_count(),
    )
}

#[cfg(test)]
mod tests {
    use super::render_ingest_output;
    use crate::runtime::CaptureIngestSummary;

    #[test]
    fn render_ingest_output_is_deterministic() {
        let output = render_ingest_output(
            "G:/My Drive/claude",
            Some("finance/inbox"),
            CaptureIngestSummary::new(1, 2, 3, 4, 5),
        );

        assert!(output.contains("capture.ingest"));
        assert!(output.contains("vault_path=G:/My Drive/claude"));
        assert!(output.contains("ingested=1"));
        assert!(output.contains("updated=2"));
        assert!(output.contains("skipped=3"));
        assert!(output.contains("conflict=4"));
        assert!(output.contains("malformed=5"));
    }
}
