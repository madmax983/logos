use comfy_table::{Cell, Color, Table};
use crossterm::style::Stylize;

use crate::app::{ReconcileRunRecord, ReconcileStatementLineRecord};

#[must_use]
pub fn render(
    month_key: Option<&str>,
    checking_account: Option<&str>,
    runs: &[ReconcileRunRecord],
    selected_run: Option<&ReconcileRunRecord>,
    selected_statement_lines: &[ReconcileStatementLineRecord],
) -> String {
    let mut lines = vec![
        "Reconciliation View".bold().cyan().to_string(),
        "═══════════════════".dark_grey().to_string(),
        format!(
            "{} {} │ {} {}",
            "Month Filter:".dark_grey(),
            month_key.unwrap_or("*"),
            "Account Filter:".dark_grey(),
            checking_account.unwrap_or("*")
        ),
        String::new(),
    ];

    let selected_id = selected_run.map(ReconcileRunRecord::run_id);

    lines.push("Runs:".bold().to_string());
    if runs.is_empty() {
        lines.push("  (no reconciliation runs)".dark_grey().to_string());
    } else {
        let mut table = Table::new();
        table.set_header(vec![
            " ",
            "Run ID",
            "Month",
            "Account",
            "Variance (Cents)",
            "Reconciled",
            "Matched TXNs",
        ]);

        for run in runs {
            let is_selected = Some(run.run_id()) == selected_id;
            let marker = if is_selected {
                Cell::new(">").fg(Color::Cyan)
            } else {
                Cell::new(" ")
            };

            let variance_val = run.variance_cents();
            let variance_cell = if variance_val == 0 {
                Cell::new(variance_val.to_string()).fg(Color::Green)
            } else {
                Cell::new(variance_val.to_string()).fg(Color::Red)
            };

            let reconciled_val = run.reconciled();
            let reconciled_cell = if reconciled_val {
                Cell::new("true").fg(Color::Green)
            } else {
                Cell::new("false").fg(Color::Red)
            };

            table.add_row(vec![
                marker,
                Cell::new(run.run_id()),
                Cell::new(run.month_key()),
                Cell::new(run.checking_account()),
                variance_cell,
                reconciled_cell,
                Cell::new(run.matched_transaction_count().to_string()),
            ]);
        }
        lines.push(table.to_string());
    }

    lines.push(String::new());
    lines.push("Statement Evidence:".bold().to_string());
    if selected_statement_lines.is_empty() {
        lines.push(
            "  (no statement lines for selected run)"
                .dark_grey()
                .to_string(),
        );
    } else {
        let mut table = Table::new();
        table.set_header(vec!["Line ID", "Timestamp", "Memo", "Amount (Cents)"]);

        for line in selected_statement_lines {
            let amount = line.amount_cents();
            let amount_cell = match amount.cmp(&0) {
                std::cmp::Ordering::Greater => Cell::new(amount.to_string()).fg(Color::Green),
                std::cmp::Ordering::Less => Cell::new(amount.to_string()).fg(Color::Red),
                std::cmp::Ordering::Equal => Cell::new(amount.to_string()),
            };

            table.add_row(vec![
                Cell::new(line.line_id()),
                Cell::new(line.statement_timestamp()),
                Cell::new(line.memo()),
                amount_cell,
            ]);
        }
        lines.push(table.to_string());
    }

    lines.join("\n")
}
