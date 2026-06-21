use crate::args::CliError;
use logos_runtime::{AppRuntime, MonthAutopilotRequest, MonthAutopilotSummary};
use logos_store::StoredFetchRunStatus;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AutopilotConfig<'a> {
    pub month_key: Option<&'a str>,
    pub checking_account: &'a str,
    pub opening_balance_cents: Option<i64>,
    pub closing_balance_cents: Option<i64>,
    pub statement_pdf: Option<&'a str>,
    pub ocr: bool,
    pub allow_variance: bool,
    pub analytics_artifact_id: Option<&'a str>,
    pub confirm_close: bool,
}

/// Handles `ledger month autopilot`.
///
/// # Errors
///
/// Returns an error when runtime initialization or workflow execution fails.
pub fn autopilot(config: &AutopilotConfig<'_>) -> Result<(), CliError> {
    let mut runtime =
        crate::runtime::init_runtime().map_err(|err| CliError::CommandRuntimeFailed {
            command: "month.autopilot".to_owned(),
            message: format!("{err}"),
        })?;
    let resolved_month_key = config.month_key.map_or_else(
        AppRuntime::<logos_store_pg::PostgresStore>::current_month_key_local,
        str::to_owned,
    );
    let mut request = MonthAutopilotRequest::new(&resolved_month_key, config.checking_account);
    if let (Some(opening_balance_cents), Some(closing_balance_cents)) =
        (config.opening_balance_cents, config.closing_balance_cents)
    {
        request = request.with_balances(opening_balance_cents, closing_balance_cents);
    }
    if let Some(path) = config.statement_pdf {
        request = request.with_statement_pdf(path);
    }
    if config.ocr {
        request = request.with_ocr(true);
    }
    if config.allow_variance {
        request = request.with_allow_variance(true);
    }
    if let Some(artifact_id) = config.analytics_artifact_id {
        request = request.with_analytics_artifact_id(artifact_id);
    }
    if config.confirm_close {
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

fn render_autopilot_output(summary: &MonthAutopilotSummary) -> String {
    let fetch_needs_attention_count = summary
        .fetch_runs()
        .iter()
        .filter(|run| run.status() == StoredFetchRunStatus::NeedsAttention)
        .count();

    let mut table = comfy_table::Table::new();
    table.load_preset(comfy_table::presets::UTF8_FULL);
    table.set_header(vec![
        "Month",
        "Account",
        "Imported",
        "Duplicates",
        "Fetch Runs",
        "Needs Attention",
        "Recon Run ID",
        "Variance",
        "Reconciled",
        "Cashflow",
        "Close ID",
        "Closed At",
    ]);

    table.add_row(vec![
        summary.month_key().to_owned(),
        summary.checking_account().to_owned(),
        summary.imported_count().to_string(),
        summary.duplicate_count().to_string(),
        summary.fetch_runs().len().to_string(),
        fetch_needs_attention_count.to_string(),
        summary.reconciliation_run().run_id().to_owned(),
        logos_core::format::currency(summary.reconciliation_run().variance_cents()),
        summary.reconciliation_run().reconciled().to_string(),
        logos_core::format::currency(summary.report().cashflow_cents()),
        summary.close().close_id().to_owned(),
        summary.close().closed_at().to_string(),
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
