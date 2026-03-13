use std::path::Path;

use crate::{
    args::CliError,
    runtime::{CaptureDraftRow, CaptureIngestSummary, CapturePromotionSummary, CliRuntime},
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
/// Returns an error when runtime initialization or capture listing fails.
pub fn list(status: Option<&str>) -> Result<(), CliError> {
    let runtime = CliRuntime::new().map_err(|err| CliError::CommandRuntimeFailed {
        command: "capture.list".to_owned(),
        message: format!("runtime initialization failed: {err}"),
    })?;
    let rows =
        runtime
            .list_capture_drafts(status)
            .map_err(|err| CliError::CommandRuntimeFailed {
                command: "capture.list".to_owned(),
                message: err.to_string(),
            })?;
    println!("{}", render_list_output(status, &rows));
    Ok(())
}

/// Handles `ledger capture show`.
///
/// # Errors
///
/// Returns an error when runtime initialization or capture lookup fails.
pub fn show(capture_id: &str) -> Result<(), CliError> {
    let runtime = CliRuntime::new().map_err(|err| CliError::CommandRuntimeFailed {
        command: "capture.show".to_owned(),
        message: format!("runtime initialization failed: {err}"),
    })?;
    let row =
        runtime
            .show_capture_draft(capture_id)
            .map_err(|err| CliError::CommandRuntimeFailed {
                command: "capture.show".to_owned(),
                message: err.to_string(),
            })?;
    println!("{}", render_show_output(&row));
    Ok(())
}

/// Handles `ledger capture promote`.
///
/// # Errors
///
pub fn promote(
    capture_id: &str,
    debit_account: Option<&str>,
    credit_account: Option<&str>,
) -> Result<(), CliError> {
    let mut runtime = CliRuntime::new().map_err(|err| CliError::CommandRuntimeFailed {
        command: "capture.promote".to_owned(),
        message: format!("runtime initialization failed: {err}"),
    })?;
    let summary = runtime
        .promote_capture_draft(capture_id, debit_account, credit_account)
        .map_err(|err| CliError::CommandRuntimeFailed {
            command: "capture.promote".to_owned(),
            message: err.to_string(),
        })?;
    println!("{}", render_promote_output(&summary));
    Ok(())
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

fn render_list_output(status: Option<&str>, rows: &[CaptureDraftRow]) -> String {
    let filter_status = status.unwrap_or("*");
    if rows.is_empty() {
        return format!("capture.list filter_status={filter_status} count=0");
    }

    let mut table = comfy_table::Table::new();
    table.load_preset(comfy_table::presets::UTF8_FULL);
    table.set_header(vec![
        "Capture ID",
        "Status",
        "Kind",
        "Amount",
        "Merchant",
        "Captured At",
    ]);

    for row in rows {
        table.add_row(vec![
            row.capture_id().to_owned(),
            row.status().to_owned(),
            row.kind().to_owned(),
            row.amount_cents().to_string(),
            row.merchant_memo().to_owned(),
            row.captured_at().to_owned(),
        ]);
    }

    format!(
        "capture.list filter_status={filter_status} count={}\n{table}",
        rows.len()
    )
}

fn render_show_output(row: &CaptureDraftRow) -> String {
    let mut table = comfy_table::Table::new();
    table.load_preset(comfy_table::presets::UTF8_FULL);
    table.set_header(vec![
        "Capture ID",
        "Status",
        "Kind",
        "Amount",
        "Currency",
        "Merchant",
        "Captured At",
        "Source Path",
        "From Hint",
        "To Hint",
        "Category Hint",
        "Promotion Txn",
        "Rejection Reason",
        "Body",
    ]);
    table.add_row(vec![
        row.capture_id().to_owned(),
        row.status().to_owned(),
        row.kind().to_owned(),
        row.amount_cents().to_string(),
        row.currency().to_owned(),
        row.merchant_memo().to_owned(),
        row.captured_at().to_owned(),
        row.source_path().to_owned(),
        row.from_account_hint().unwrap_or("-").to_owned(),
        row.to_account_hint().unwrap_or("-").to_owned(),
        row.category_hint().unwrap_or("-").to_owned(),
        row.promotion_txn_id().unwrap_or("-").to_owned(),
        row.rejection_reason().unwrap_or("-").to_owned(),
        if row.body_note().is_empty() {
            "-".to_owned()
        } else {
            row.body_note().to_owned()
        },
    ]);
    table.to_string()
}

fn render_promote_output(summary: &CapturePromotionSummary) -> String {
    format!(
        "capture.promote capture_id={} transaction_id={} debit_account={} credit_account={}",
        summary.capture_id(),
        summary.transaction_id(),
        summary.debit_account(),
        summary.credit_account(),
    )
}

#[cfg(test)]
mod tests {
    use super::{
        render_ingest_output, render_list_output, render_promote_output, render_show_output,
    };
    use crate::runtime::{CaptureDraftRow, CaptureIngestSummary, CapturePromotionSummary};
    use logos_core::TransactionId;

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

    #[test]
    fn render_list_output_is_deterministic() {
        let row = CaptureDraftRow::new_for_tests(
            "cap-1",
            "G:/My Drive/claude/finance/inbox/2026/03/cap-1.md",
            "sha256:abc",
            "2026-03-11T18:42:05Z",
            "expense",
            1_284,
            "USD",
            "Tacos El Rey",
            Some("liabilities:amex:gold"),
            None,
            Some("expenses:food:dining"),
            "Team dinner",
            "inbox",
            Some("expenses:food:dining"),
            Some("liabilities:amex:gold"),
            None,
            None,
        );
        let output = render_list_output(Some("inbox"), &[row]);

        assert!(output.contains("capture.list filter_status=inbox"));
        assert!(output.contains("cap-1"));
        assert!(output.contains("expense"));
        assert!(output.contains("Tacos El Rey"));
    }

    #[test]
    fn render_show_output_is_deterministic() {
        let row = CaptureDraftRow::new_for_tests(
            "cap-1",
            "G:/My Drive/claude/finance/inbox/2026/03/cap-1.md",
            "sha256:abc",
            "2026-03-11T18:42:05Z",
            "expense",
            1_284,
            "USD",
            "Tacos El Rey",
            Some("liabilities:amex:gold"),
            None,
            Some("expenses:food:dining"),
            "Team dinner",
            "inbox",
            Some("expenses:food:dining"),
            Some("liabilities:amex:gold"),
            None,
            None,
        );
        let output = render_show_output(&row);

        assert!(output.contains("cap-1"));
        assert!(output.contains("Tacos El Rey"));
        assert!(output.contains("Team dinner"));
        assert!(output.contains("liabilities:amex:gold"));
    }

    #[test]
    fn render_promote_output_is_deterministic() {
        let summary = CapturePromotionSummary::new(
            "cap-1",
            &TransactionId::new("txn-1").expect("txn"),
            "expenses:food:dining",
            "liabilities:amex:gold",
        );
        let output = render_promote_output(&summary);

        assert!(output.contains("capture.promote"));
        assert!(output.contains("capture_id=cap-1"));
        assert!(output.contains("transaction_id=txn-1"));
        assert!(output.contains("debit_account=expenses:food:dining"));
        assert!(output.contains("credit_account=liabilities:amex:gold"));
    }
}
