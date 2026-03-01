use crate::{
    args::CliError,
    runtime::{CliRuntime, MonthAutopilotRequest, MonthAutopilotSummary},
};

/// Handles `ledger month autopilot`.
///
/// # Errors
///
/// Returns an error when runtime initialization or workflow execution fails.
#[allow(clippy::too_many_arguments)]
pub fn autopilot(
    month_key: Option<&str>,
    checking_account: &str,
    opening_balance_cents: i64,
    closing_balance_cents: i64,
    statement_pdf: Option<&str>,
    ocr: bool,
    allow_variance: bool,
    analytics_artifact_id: Option<&str>,
    confirm_close: bool,
) -> Result<(), CliError> {
    let mut runtime = CliRuntime::new().map_err(|err| CliError::CommandRuntimeFailed {
        command: "month.autopilot".to_owned(),
        message: format!("runtime initialization failed: {err}"),
    })?;
    let resolved_month_key =
        month_key.map_or_else(CliRuntime::current_month_key_local, str::to_owned);
    let mut request = MonthAutopilotRequest::new(
        &resolved_month_key,
        checking_account,
        opening_balance_cents,
        closing_balance_cents,
    );
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

fn render_autopilot_output(summary: &MonthAutopilotSummary) -> String {
    format!(
        "month.autopilot month={} checking_account={} imported_count={} duplicate_count={} run_id={} variance_cents={} reconciled={} report_cashflow_cents={} close_id={} closed_at_us={}",
        summary.month_key(),
        summary.checking_account(),
        summary.imported_count(),
        summary.duplicate_count(),
        summary.reconciliation_run().run_id(),
        summary.reconciliation_run().variance_cents(),
        summary.reconciliation_run().reconciled(),
        summary.report().cashflow_cents(),
        summary.close().close_id(),
        summary.close().closed_at().wallclock()
    )
}

#[cfg(test)]
mod tests {
    use super::render_autopilot_output;
    use crate::runtime::{MonthAutopilotSummary, MonthReport};
    use logos_store_aletheia::{StoredMonthClose, StoredReconciliationRun};

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
            1_700_000_123_i64.into(),
        );
        let close = StoredMonthClose::new(
            "close-2",
            "2026-04",
            "assets:checking",
            "recon-5",
            Some("artifact-2"),
            1_700_000_456_i64.into(),
        );
        let summary = MonthAutopilotSummary::new(
            "2026-04",
            "assets:checking",
            2,
            0,
            run,
            MonthReport::new(107_500, 10_000, 2_500, 7_500),
            close,
        );

        let output = render_autopilot_output(&summary);
        assert_eq!(
            output,
            "month.autopilot month=2026-04 checking_account=assets:checking imported_count=2 duplicate_count=0 run_id=recon-5 variance_cents=0 reconciled=true report_cashflow_cents=7500 close_id=close-2 closed_at_us=1700000456"
        );
    }
}
