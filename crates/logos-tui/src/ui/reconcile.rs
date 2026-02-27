use crate::app::{ReconcileRunRecord, ReconcileStatementLineRecord};

#[must_use]
pub fn render(
    month_key: Option<&str>,
    checking_account: Option<&str>,
    runs: &[ReconcileRunRecord],
    selected_run: Option<&ReconcileRunRecord>,
    selected_statement_lines: &[ReconcileStatementLineRecord],
) -> String {
    let selected_run_id = selected_run.map_or("-", ReconcileRunRecord::run_id);
    let mut lines = vec![format!(
        "Reconciliation View | filter_month={} filter_account={} runs={} selected_run={selected_run_id}",
        month_key.unwrap_or("*"),
        checking_account.unwrap_or("*"),
        runs.len(),
    )];

    lines.push("Runs:".to_owned());
    if runs.is_empty() {
        lines.push("  (no reconciliation runs)".to_owned());
    } else {
        let selected_id = selected_run.map(ReconcileRunRecord::run_id);
        for run in runs {
            let marker = if Some(run.run_id()) == selected_id {
                '*'
            } else {
                ' '
            };
            lines.push(format!(
                "  {marker} run_id={} month={} account={} variance_cents={} reconciled={} matched_transactions={}",
                run.run_id(),
                run.month_key(),
                run.checking_account(),
                run.variance_cents(),
                run.reconciled(),
                run.matched_transaction_count(),
            ));
        }
    }

    lines.push("Statement Evidence:".to_owned());
    if selected_statement_lines.is_empty() {
        lines.push("  (no statement lines for selected run)".to_owned());
    } else {
        for line in selected_statement_lines {
            lines.push(format!(
                "  line_id={} ts={} memo={} amount_cents={}",
                line.line_id(),
                line.statement_timestamp(),
                line.memo(),
                line.amount_cents(),
            ));
        }
    }

    lines.join("\n")
}
