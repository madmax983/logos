use crate::args::CliError;
use logos_runtime::{AppRuntime, MonthAutopilotRequest, MonthAutopilotSummary};
use logos_store::StoredFetchRunStatus;

/// Handles `ledger month autopilot`.
///
/// # Errors
///
/// Returns an error when runtime initialization or workflow execution fails.
#[allow(clippy::too_many_arguments)]
pub fn autopilot(
    month_key: Option<&str>,
    checking_account: &str,
    opening_balance_cents: Option<i64>,
    closing_balance_cents: Option<i64>,
    statement_pdf: Option<&str>,
    ocr: bool,
    allow_variance: bool,
    analytics_artifact_id: Option<&str>,
    confirm_close: bool,
) -> Result<(), CliError> {
    let mut runtime =
        crate::runtime::init_runtime().map_err(|err| CliError::CommandRuntimeFailed {
            command: "month.autopilot".to_owned(),
            message: format!("{err}"),
        })?;
    let resolved_month_key = month_key.map_or_else(
        AppRuntime::<logos_store_pg::PostgresStore>::current_month_key_local,
        str::to_owned,
    );
    let mut request = MonthAutopilotRequest::new(&resolved_month_key, checking_account);
    if let (Some(opening_balance_cents), Some(closing_balance_cents)) =
        (opening_balance_cents, closing_balance_cents)
    {
        request = request.with_balances(opening_balance_cents, closing_balance_cents);
    }
    if let Some(path) = statement_pdf {
        request = request.with_statement_pdf(path);
    }
    if ocr {
        request = request.with_ocr(true);
    }
    if allow_variance {
        request = request.with_allow_variance(true);
    }
    if let Some(artifact_id) = analytics_artifact_id {
        request = request.with_analytics_artifact_id(artifact_id);
    }
    if confirm_close {
        request = request.with_confirm_close(true);
    }

    let summary =
        runtime
            .run_month_autopilot(&request)
            .map_err(|err| CliError::CommandRuntimeFailed {
                command: "month.autopilot".to_owned(),
                message: err.to_string(),
            })?;
    println!("{}", render_autopilot_output(&summary));
    Ok(())
}

use comfy_table::{Attribute, Cell, Color};

fn render_autopilot_output(summary: &MonthAutopilotSummary) -> String {
    let fetch_needs_attention_count = summary
        .fetch_runs()
        .iter()
        .filter(|run| run.status() == StoredFetchRunStatus::NeedsAttention)
        .count();

    let mut table = comfy_table::Table::new();
    table.load_preset(comfy_table::presets::UTF8_FULL);
    table.set_header(vec![
        Cell::new("Month")
            .fg(Color::Cyan)
            .add_attribute(Attribute::Bold),
        Cell::new("Account")
            .fg(Color::Cyan)
            .add_attribute(Attribute::Bold),
        Cell::new("Imported")
            .fg(Color::Cyan)
            .add_attribute(Attribute::Bold),
        Cell::new("Duplicates")
            .fg(Color::Cyan)
            .add_attribute(Attribute::Bold),
        Cell::new("Fetch Runs")
            .fg(Color::Cyan)
            .add_attribute(Attribute::Bold),
        Cell::new("Needs Attention")
            .fg(Color::Cyan)
            .add_attribute(Attribute::Bold),
        Cell::new("Recon Run ID")
            .fg(Color::Cyan)
            .add_attribute(Attribute::Bold),
        Cell::new("Variance")
            .fg(Color::Cyan)
            .add_attribute(Attribute::Bold),
        Cell::new("Reconciled")
            .fg(Color::Cyan)
            .add_attribute(Attribute::Bold),
        Cell::new("Cashflow")
            .fg(Color::Cyan)
            .add_attribute(Attribute::Bold),
        Cell::new("Close ID")
            .fg(Color::Cyan)
            .add_attribute(Attribute::Bold),
        Cell::new("Closed At")
            .fg(Color::Cyan)
            .add_attribute(Attribute::Bold),
    ]);

    let attention_color = if fetch_needs_attention_count > 0 {
        Color::Red
    } else {
        Color::Green
    };
    let variance_cents = summary.reconciliation_run().variance_cents();
    let variance_color = if variance_cents != 0 {
        Color::Red
    } else {
        Color::Green
    };
    let reconciled_color = if summary.reconciliation_run().reconciled() {
        Color::Green
    } else {
        Color::Red
    };
    let cashflow_cents = summary.report().cashflow_cents();
    let cashflow_color = if cashflow_cents >= 0 {
        Color::Green
    } else {
        Color::Red
    };

    table.add_row(vec![
        Cell::new(summary.month_key()),
        Cell::new(summary.checking_account()),
        Cell::new(summary.imported_count()),
        Cell::new(summary.duplicate_count()),
        Cell::new(summary.fetch_runs().len()),
        Cell::new(fetch_needs_attention_count).fg(attention_color),
        Cell::new(summary.reconciliation_run().run_id()),
        Cell::new(logos_core::format::currency(variance_cents)).fg(variance_color),
        Cell::new(summary.reconciliation_run().reconciled()).fg(reconciled_color),
        Cell::new(logos_core::format::currency(cashflow_cents)).fg(cashflow_color),
        Cell::new(summary.close().close_id()),
        Cell::new(summary.close().closed_at()),
    ]);

    format!("month.autopilot\n{table}")
}

#[cfg(test)]
mod tests {
    use super::render_autopilot_output;
    use logos_runtime::{MonthAutopilotSummary, MonthReport};
    use logos_store::{
        StoredFetchArtifactFormat, StoredFetchRun, StoredFetchRunStatus, StoredMonthClose,
        StoredReconciliationRun,
    };

    #[test]
    fn render_autopilot_output_is_deterministic() {
        let run = StoredReconciliationRun::new(
            "recon-5",
            "2026-04",
            "assets:checking",
            100_000,
            7_500,
            107_500,
            107_500,
            0,
            true,
            2,
            2,
            10_000,
            2_500,
            1_700_000_123_i64,
        );
        let close = StoredMonthClose::new(
            "close-2",
            "2026-04",
            "assets:checking",
            "recon-5",
            Some("artifact-2"),
            1_700_000_456_i64,
        );
        let fetch_run = StoredFetchRun::new(
            "fetch-1",
            "pcu:checking",
            "provident-credit-union",
            "assets:checking",
            "2026-04",
            StoredFetchRunStatus::Downloaded,
            Some("C:\\statements\\pcu-2026-04.pdf"),
            Some(StoredFetchArtifactFormat::Pdf),
            Some(100_000),
            Some(107_500),
            None,
            1_700_000_100_i64,
        );
        let summary = MonthAutopilotSummary::new(
            "2026-04",
            "assets:checking",
            2,
            0,
            vec![fetch_run],
            run,
            MonthReport::new(107_500, 10_000, 2_500, 7_500),
            close,
        );

        let output = render_autopilot_output(&summary);
        let output = String::from_utf8(strip_ansi_escapes::strip(&output)).unwrap();
        assert!(output.contains("month.autopilot"));
        assert!(output.contains("2026-04"));
        assert!(output.contains("assets:checking"));
        assert!(output.contains("recon-5"));
        assert!(output.contains("close-2"));
        assert!(output.contains("$0.00"));
        assert!(output.contains("true"));
        assert!(output.contains("$75.00"));
        assert!(output.contains("1700000456"));
    }
}
