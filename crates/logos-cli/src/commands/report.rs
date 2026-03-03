use crate::args::CliError;
use logos_app::{CliRuntime, MonthReport};

trait ReportRuntime {
    fn month_report_for(&self, checking_account: &str, month_key: &str) -> MonthReport;
}

impl ReportRuntime for CliRuntime {
    fn month_report_for(&self, checking_account: &str, month_key: &str) -> MonthReport {
        Self::month_report_for(self, checking_account, month_key)
    }
}

/// Handles `ledger report month`.
///
/// # Errors
///
/// Returns an error when runtime initialization fails.
pub fn month(checking_account: &str, month_key: Option<&str>) -> Result<(), CliError> {
    let runtime = CliRuntime::new().map_err(|err: logos_app::RuntimeError| {
        CliError::CommandRuntimeFailed {
            command: "report.month".to_owned(),
            message: format!("runtime initialization failed: {err}"),
        }
    })?;
    let resolved_month_key =
        month_key.map_or_else(CliRuntime::current_month_key_local, str::to_owned);
    let output = render_month_output(&runtime, checking_account, &resolved_month_key);
    println!("{output}");
    Ok(())
}

fn render_month_output(
    runtime: &impl ReportRuntime,
    checking_account: &str,
    month_key: &str,
) -> String {
    let report = runtime.month_report_for(checking_account, month_key);
    format!(
        "report.month month={month_key} checking_account={checking_account} checking_balance_cents={} income_cents={} expense_cents={} cashflow_cents={}",
        report.checking_balance_cents(),
        report.income_cents(),
        report.expense_cents(),
        report.cashflow_cents()
    )
}

#[cfg(test)]
mod tests {
    use super::{ReportRuntime, render_month_output};
    use logos_app::MonthReport;

    struct FakeReportRuntime {
        report: MonthReport,
    }

    impl ReportRuntime for FakeReportRuntime {
        fn month_report_for(&self, _checking_account: &str, _month_key: &str) -> MonthReport {
            self.report
        }
    }

    #[test]
    fn render_month_output_is_deterministic() {
        let runtime = FakeReportRuntime {
            report: MonthReport::new(7_500, 10_000, 2_500, 7_500),
        };

        let output = render_month_output(&runtime, "assets:checking", "2026-03");

        assert_eq!(
            output,
            "report.month month=2026-03 checking_account=assets:checking checking_balance_cents=7500 income_cents=10000 expense_cents=2500 cashflow_cents=7500"
        );
    }
}
