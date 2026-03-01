use crate::{args::CliError, runtime::CliRuntime};
use logos_store_aletheia::StoredReconciliationRun;

/// Handles `ledger reconcile month`.
///
/// # Errors
///
/// Returns an error when runtime initialization fails.
pub fn month(
    checking_account: &str,
    month_key: Option<&str>,
    opening_balance_cents: i64,
    closing_balance_cents: i64,
) -> Result<(), CliError> {
    let mut runtime = CliRuntime::new().map_err(|err| CliError::CommandRuntimeFailed {
        command: "reconcile.month".to_owned(),
        message: format!("runtime initialization failed: {err}"),
    })?;
    let resolved_month_key =
        month_key.map_or_else(CliRuntime::current_month_key_local, str::to_owned);
    let run = runtime
        .reconcile_and_persist_month_for(
            checking_account,
            &resolved_month_key,
            opening_balance_cents,
            closing_balance_cents,
        )
        .map_err(|err| CliError::CommandRuntimeFailed {
            command: "reconcile.month".to_owned(),
            message: err.to_string(),
        })?;
    let output = render_month_output(
        checking_account,
        &resolved_month_key,
        opening_balance_cents,
        &run,
    );
    println!("{output}");
    Ok(())
}

/// Handles `ledger reconcile list`.
///
/// # Errors
///
/// Returns an error when runtime initialization fails.
pub fn list(month_key: Option<&str>, checking_account: Option<&str>) -> Result<(), CliError> {
    let runtime = CliRuntime::new().map_err(|err| CliError::CommandRuntimeFailed {
        command: "reconcile.list".to_owned(),
        message: format!("runtime initialization failed: {err}"),
    })?;
    let runs = runtime.list_reconciliation_runs(month_key, checking_account);
    println!("{}", render_list_output(month_key, checking_account, &runs));
    Ok(())
}

/// Handles `ledger reconcile show`.
///
/// # Errors
///
/// Returns an error when runtime initialization fails.
pub fn show(run_id: &str) -> Result<(), CliError> {
    let runtime = CliRuntime::new().map_err(|err| CliError::CommandRuntimeFailed {
        command: "reconcile.show".to_owned(),
        message: format!("runtime initialization failed: {err}"),
    })?;
    let Some(run) = runtime.reconciliation_run(run_id) else {
        return Err(CliError::CommandRuntimeFailed {
            command: "reconcile.show".to_owned(),
            message: format!("reconciliation run '{run_id}' not found"),
        });
    };
    println!("{}", render_show_output(&run));
    Ok(())
}

fn render_month_output(
    checking_account: &str,
    month_key: &str,
    opening_balance_cents: i64,
    run: &StoredReconciliationRun,
) -> String {
    format!(
        "reconcile.month run_id={} month={month_key} checking_account={checking_account} opening_balance_cents={opening_balance_cents} ledger_delta_cents={} expected_closing_balance_cents={} statement_closing_balance_cents={} variance_cents={} reconciled={} matched_postings={} matched_transactions={} inflow_cents={} outflow_cents={} created_at_us={}",
        run.run_id(),
        run.ledger_delta_cents(),
        run.expected_closing_balance_cents(),
        run.statement_closing_balance_cents(),
        run.variance_cents(),
        run.reconciled(),
        run.matched_postings(),
        run.matched_transaction_count(),
        run.inflow_cents(),
        run.outflow_cents(),
        run.created_at().wallclock()
    )
}

fn render_show_output(run: &StoredReconciliationRun) -> String {
    format!(
        "reconcile.show run_id={} month={} checking_account={} opening_balance_cents={} ledger_delta_cents={} expected_closing_balance_cents={} statement_closing_balance_cents={} variance_cents={} reconciled={} matched_postings={} matched_transactions={} inflow_cents={} outflow_cents={} created_at_us={}",
        run.run_id(),
        run.month_key(),
        run.checking_account(),
        run.opening_balance_cents(),
        run.ledger_delta_cents(),
        run.expected_closing_balance_cents(),
        run.statement_closing_balance_cents(),
        run.variance_cents(),
        run.reconciled(),
        run.matched_postings(),
        run.matched_transaction_count(),
        run.inflow_cents(),
        run.outflow_cents(),
        run.created_at().wallclock()
    )
}

fn render_list_output(
    month_key: Option<&str>,
    checking_account: Option<&str>,
    runs: &[StoredReconciliationRun],
) -> String {
    let filter_month = month_key.unwrap_or("*");
    let filter_account = checking_account.unwrap_or("*");
    if runs.is_empty() {
        return format!(
            "reconcile.list filter_month={filter_month} filter_checking_account={filter_account} count=0"
        );
    }

    let mut lines = Vec::with_capacity(runs.len() + 1);
    lines.push(format!(
        "reconcile.list filter_month={filter_month} filter_checking_account={filter_account} count={}",
        runs.len()
    ));
    for run in runs {
        lines.push(format!(
            "reconcile.item run_id={} month={} checking_account={} variance_cents={} reconciled={} matched_transactions={} created_at_us={}",
            run.run_id(),
            run.month_key(),
            run.checking_account(),
            run.variance_cents(),
            run.reconciled(),
            run.matched_transaction_count(),
            run.created_at().wallclock()
        ));
    }
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::{render_list_output, render_month_output, render_show_output};
    use logos_store_aletheia::StoredReconciliationRun;

    #[test]
    fn render_month_output_is_deterministic() {
        let run = StoredReconciliationRun::new(
            "recon-7",
            "2026-03",
            "assets:checking",
            100_000,
            7_500,
            107_500,
            106_000,
            -1_500,
            false,
            2,
            2,
            10_000,
            2_500,
            1_700_000_111_i64.into(),
        );
        let output = render_month_output("assets:checking", "2026-03", 100_000, &run);

        assert_eq!(
            output,
            "reconcile.month run_id=recon-7 month=2026-03 checking_account=assets:checking opening_balance_cents=100000 ledger_delta_cents=7500 expected_closing_balance_cents=107500 statement_closing_balance_cents=106000 variance_cents=-1500 reconciled=false matched_postings=2 matched_transactions=2 inflow_cents=10000 outflow_cents=2500 created_at_us=1700000111"
        );
    }

    #[test]
    fn render_show_output_is_deterministic() {
        let run = StoredReconciliationRun::new(
            "recon-8",
            "2026-04",
            "assets:checking",
            200_000,
            12_000,
            212_000,
            212_500,
            500,
            false,
            3,
            2,
            15_000,
            3_000,
            1_700_000_222_i64.into(),
        );
        let output = render_show_output(&run);
        assert_eq!(
            output,
            "reconcile.show run_id=recon-8 month=2026-04 checking_account=assets:checking opening_balance_cents=200000 ledger_delta_cents=12000 expected_closing_balance_cents=212000 statement_closing_balance_cents=212500 variance_cents=500 reconciled=false matched_postings=3 matched_transactions=2 inflow_cents=15000 outflow_cents=3000 created_at_us=1700000222"
        );
    }

    #[test]
    fn render_list_output_is_deterministic() {
        let runs = vec![
            StoredReconciliationRun::new(
                "recon-9",
                "2026-05",
                "assets:checking",
                100_000,
                9_000,
                109_000,
                109_000,
                0,
                true,
                2,
                2,
                11_000,
                2_000,
                1_700_000_333_i64.into(),
            ),
            StoredReconciliationRun::new(
                "recon-10",
                "2026-05",
                "assets:checking",
                109_000,
                4_000,
                113_000,
                112_500,
                -500,
                false,
                1,
                1,
                4_000,
                0,
                1_700_000_444_i64.into(),
            ),
        ];

        let output = render_list_output(Some("2026-05"), Some("assets:checking"), &runs);
        assert_eq!(
            output,
            "reconcile.list filter_month=2026-05 filter_checking_account=assets:checking count=2\nreconcile.item run_id=recon-9 month=2026-05 checking_account=assets:checking variance_cents=0 reconciled=true matched_transactions=2 created_at_us=1700000333\nreconcile.item run_id=recon-10 month=2026-05 checking_account=assets:checking variance_cents=-500 reconciled=false matched_transactions=1 created_at_us=1700000444"
        );
    }
}
