#![allow(clippy::cast_precision_loss)]
use crate::{
    args::CliError,
    runtime::{CliRuntime, MonthReport},
};

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
    let runtime = CliRuntime::new().map_err(|err| CliError::CommandRuntimeFailed {
        command: "report.month".to_owned(),
        message: format!("runtime initialization failed: {err}"),
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

    let mut table = comfy_table::Table::new();
    table.load_preset(comfy_table::presets::UTF8_FULL);
    table.set_header(vec![
        "Month",
        "Checking Account",
        "Balance",
        "Income",
        "Expense",
        "Cashflow",
    ]);
    table.add_row(vec![
        month_key.to_owned(),
        checking_account.to_owned(),
        format!("${:.2}", (report.checking_balance_cents() as f64) / 100.0),
        format!("${:.2}", (report.income_cents() as f64) / 100.0),
        format!("${:.2}", (report.expense_cents() as f64) / 100.0),
        format!("${:.2}", (report.cashflow_cents() as f64) / 100.0),
    ]);

    format!("report.month\n{table}")
}

#[cfg(test)]
mod tests {
    use super::{ReportRuntime, render_month_output};
    use crate::runtime::MonthReport;

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

        assert!(output.contains("report.month"));
        assert!(output.contains("2026-03"));
        assert!(output.contains("assets:checking"));
        assert!(output.contains("$75.00"));
        assert!(output.contains("$100.00"));
        assert!(output.contains("$25.00"));
        assert!(output.contains("$75.00"));
    }
}
