#![allow(clippy::cast_precision_loss)]
use crate::args::CliError;
use logos_runtime::{AppRuntime, MonthReport};

trait ReportRuntime {
    fn month_report_for(&self, checking_account: &str, month_key: &str) -> MonthReport;
}

impl ReportRuntime for AppRuntime<logos_store_pg::PostgresStore> {
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
    let runtime = crate::runtime::init_runtime().map_err(|err| CliError::CommandRuntimeFailed {
        command: "report.month".to_owned(),
        message: format!("{err}"),
    })?;
    let resolved_month_key =
        month_key.map_or_else(AppRuntime::<logos_store_pg::PostgresStore>::current_month_key_local, str::to_owned);
    let output = render_month_output(&runtime, checking_account, &resolved_month_key);
    println!("{output}");
    Ok(())
}

use comfy_table::{Attribute, Cell, Color};

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

    let cashflow_cents = report.cashflow_cents();
    let cashflow_color = if cashflow_cents >= 0 {
        Color::Green
    } else {
        Color::Red
    };
    let cashflow_cell = Cell::new(format!("${:.2}", (cashflow_cents as f64) / 100.0))
        .fg(cashflow_color)
        .add_attribute(Attribute::Bold);

    table.add_row(vec![
        Cell::new(month_key.to_string()),
        Cell::new(checking_account.to_string()),
        Cell::new(format!(
            "${:.2}",
            (report.checking_balance_cents() as f64) / 100.0
        )),
        Cell::new(format!("${:.2}", (report.income_cents() as f64) / 100.0)),
        Cell::new(format!("${:.2}", (report.expense_cents() as f64) / 100.0)),
        cashflow_cell,
    ]);

    format!("{table}")
}

#[cfg(test)]
mod tests {
    use super::{ReportRuntime, render_month_output};
    use logos_runtime::MonthReport;

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

        assert!(output.contains("2026-03"));
        assert!(output.contains("assets:checking"));
        assert!(output.contains("$75.00"));
        assert!(output.contains("$100.00"));
        assert!(output.contains("$25.00"));
        assert!(output.contains("$75.00"));
    }
}
