use crate::{args::CliError, runtime::CliRuntime};
use logos_store_aletheia::model::StoredReconciliationRun;

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
    let mut table = comfy_table::Table::new();
    table.load_preset(comfy_table::presets::UTF8_FULL);

    table.set_header(vec![
        "Run ID",
        "Month",
        "Account",
        "Opening",
        "Ledger Δ",
        "Expected Closing",
        "Statement Closing",
        "Variance",
        "Reconciled",
        "Matched Postings",
        "Matched Txns",
        "Inflow",
        "Outflow",
        "Created At",
    ]);
    table.add_row(vec![
        run.run_id().to_owned(),
        month_key.to_owned(),
        checking_account.to_owned(),
        opening_balance_cents.to_string(),
        run.ledger_delta_cents().to_string(),
        run.expected_closing_balance_cents().to_string(),
        run.statement_closing_balance_cents().to_string(),
        run.variance_cents().to_string(),
        run.reconciled().to_string(),
        run.matched_postings().to_string(),
        run.matched_transaction_count().to_string(),
        run.inflow_cents().to_string(),
        run.outflow_cents().to_string(),
        run.created_at().wallclock().to_string(),
    ]);
    table.to_string()
}

fn render_show_output(run: &StoredReconciliationRun) -> String {
    let mut table = comfy_table::Table::new();
    table.load_preset(comfy_table::presets::UTF8_FULL);

    table.set_header(vec![
        "Run ID",
        "Month",
        "Account",
        "Opening",
        "Ledger Δ",
        "Expected Closing",
        "Statement Closing",
        "Variance",
        "Reconciled",
        "Matched Postings",
        "Matched Txns",
        "Inflow",
        "Outflow",
        "Created At",
    ]);
    table.add_row(vec![
        run.run_id().to_owned(),
        run.month_key().to_owned(),
        run.checking_account().to_owned(),
        run.opening_balance_cents().to_string(),
        run.ledger_delta_cents().to_string(),
        run.expected_closing_balance_cents().to_string(),
        run.statement_closing_balance_cents().to_string(),
        run.variance_cents().to_string(),
        run.reconciled().to_string(),
        run.matched_postings().to_string(),
        run.matched_transaction_count().to_string(),
        run.inflow_cents().to_string(),
        run.outflow_cents().to_string(),
        run.created_at().wallclock().to_string(),
    ]);
    table.to_string()
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

    let mut table = comfy_table::Table::new();
    table.load_preset(comfy_table::presets::UTF8_FULL);

    table.set_header(vec![
        "Run ID",
        "Month",
        "Account",
        "Variance",
        "Reconciled",
        "Matched Txns",
        "Created At",
    ]);

    for run in runs {
        table.add_row(vec![
            run.run_id().to_owned(),
            run.month_key().to_owned(),
            run.checking_account().to_owned(),
            run.variance_cents().to_string(),
            run.reconciled().to_string(),
            run.matched_transaction_count().to_string(),
            run.created_at().wallclock().to_string(),
        ]);
    }

    format!(
        "reconcile.list filter_month={filter_month} filter_checking_account={filter_account} count={}\n{table}",
        runs.len()
    )
}

#[cfg(test)]
mod tests {
    use super::{render_list_output, render_month_output, render_show_output};
    use logos_store_aletheia::model::StoredReconciliationRun;

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

        assert!(output.contains("recon-7"));
        assert!(output.contains("2026-03"));
        assert!(output.contains("assets:checking"));
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
        assert!(output.contains("recon-8"));
        assert!(output.contains("2026-04"));
        assert!(output.contains("assets:checking"));
    }

    #[test]
    fn render_list_output_is_deterministic() {
        let runs = vec![
            StoredReconciliationRun::new(
                "recon-9",
                "2026-05",
                "assets:checking",
                105_000,
                5_000,
                110_000,
                110_000,
                0,
                true,
                2,
                2,
                7_000,
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
        assert!(output.contains("recon-9"));
        assert!(output.contains("recon-10"));
        assert!(output.contains("2026-05"));
        assert!(output.contains("assets:checking"));
    }
}
