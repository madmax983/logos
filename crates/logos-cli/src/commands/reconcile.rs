use crate::args::CliError;
use comfy_table::{Attribute, Cell, Color};
use logos_core::us_timestamp;
use logos_runtime::AppRuntime;
use logos_store::StoredReconciliationRun;

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
    let mut runtime =
        crate::runtime::init_runtime().map_err(|err| CliError::CommandRuntimeFailed {
            command: "reconcile.month".to_owned(),
            message: format!("{err}"),
        })?;
    let resolved_month_key = month_key.map_or_else(
        AppRuntime::<logos_store_pg::PostgresStore>::current_month_key_local,
        str::to_owned,
    );
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
    let runtime = crate::runtime::init_runtime().map_err(|err| CliError::CommandRuntimeFailed {
        command: "reconcile.list".to_owned(),
        message: format!("{err}"),
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
    let runtime = crate::runtime::init_runtime().map_err(|err| CliError::CommandRuntimeFailed {
        command: "reconcile.show".to_owned(),
        message: format!("{err}"),
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
    set_run_headers(&mut table);
    table.add_row(add_run_row(
        month_key,
        checking_account,
        opening_balance_cents,
        run,
    ));
    table.to_string()
}

fn render_show_output(run: &StoredReconciliationRun) -> String {
    let mut table = comfy_table::Table::new();
    table.load_preset(comfy_table::presets::UTF8_FULL);
    set_run_headers(&mut table);
    table.add_row(add_run_row(
        run.month_key(),
        run.checking_account(),
        run.opening_balance_cents(),
        run,
    ));
    table.to_string()
}

fn render_list_output(
    _month_key: Option<&str>,
    _checking_account: Option<&str>,
    runs: &[StoredReconciliationRun],
) -> String {
    if runs.is_empty() {
        return "No reconciliation runs found for the given filters.".to_string();
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
        let variance_cell = if run.variance_cents() == 0 {
            Cell::new("$0.00").fg(Color::Green)
        } else {
            Cell::new(logos_core::currency(run.variance_cents()))
                .fg(Color::Red)
                .add_attribute(Attribute::Bold)
        };

        let reconciled_cell = if run.reconciled() {
            Cell::new("true").fg(Color::Green)
        } else {
            Cell::new("false").fg(Color::Red)
        };

        table.add_row(vec![
            Cell::new(run.run_id()).fg(Color::DarkGrey),
            Cell::new(run.month_key()),
            Cell::new(run.checking_account()),
            variance_cell,
            reconciled_cell,
            Cell::new(run.matched_transaction_count()),
            Cell::new(us_timestamp(run.created_at())).fg(Color::DarkGrey),
        ]);
    }

    format!("{table}")
}

fn set_run_headers(table: &mut comfy_table::Table) {
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
}

fn add_run_row(
    month_key: &str,
    checking_account: &str,
    opening_balance_cents: i64,
    run: &StoredReconciliationRun,
) -> Vec<Cell> {
    let variance_cell = if run.variance_cents() == 0 {
        Cell::new("$0.00").fg(Color::Green)
    } else {
        Cell::new(logos_core::currency(run.variance_cents()))
            .fg(Color::Red)
            .add_attribute(Attribute::Bold)
    };

    let reconciled_cell = if run.reconciled() {
        Cell::new("true").fg(Color::Green)
    } else {
        Cell::new("false").fg(Color::Red)
    };

    vec![
        Cell::new(run.run_id()).fg(Color::DarkGrey),
        Cell::new(month_key),
        Cell::new(checking_account),
        Cell::new(logos_core::currency(opening_balance_cents)),
        Cell::new(logos_core::currency(run.ledger_delta_cents())),
        Cell::new(logos_core::currency(
            run.expected_closing_balance_cents(),
        )),
        Cell::new(logos_core::currency(
            run.statement_closing_balance_cents(),
        )),
        variance_cell,
        reconciled_cell,
        Cell::new(run.matched_postings()),
        Cell::new(run.matched_transaction_count()),
        Cell::new(logos_core::currency(run.inflow_cents())).fg(Color::Green),
        Cell::new(logos_core::currency(run.outflow_cents())).fg(Color::Red),
        Cell::new(us_timestamp(run.created_at())).fg(Color::DarkGrey),
    ]
}

#[cfg(test)]
mod tests {
    use super::{render_list_output, render_month_output, render_show_output};
    use logos_store::StoredReconciliationRun;

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
            1_700_000_111_i64,
        );
        let output = render_month_output("assets:checking", "2026-03", 100_000, &run);

        assert!(output.contains("recon-7"));
        assert!(output.contains("$1,000.00"));
        assert!(output.contains("$75.00"));
        assert!(output.contains("$1,075.00"));
        assert!(output.contains("$1,060.00"));
        assert!(output.contains("-$15.00"));
        assert!(output.contains("$100.00"));
        assert!(output.contains("$25.00"));
        assert!(output.contains("1970-01-01 00:28:20 UTC"));
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
            1_700_000_222_i64,
        );
        let output = render_show_output(&run);

        assert!(output.contains("recon-8"));
        assert!(output.contains("$2,000.00"));
        assert!(output.contains("$120.00"));
        assert!(output.contains("$2,120.00"));
        assert!(output.contains("$2,125.00"));
        assert!(output.contains("$5.00"));
        assert!(output.contains("$150.00"));
        assert!(output.contains("$30.00"));
        assert!(output.contains("1970-01-01 00:28:20 UTC"));
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
                1_700_000_333_i64,
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
                1_700_000_444_i64,
            ),
        ];

        let output = render_list_output(Some("2026-05"), Some("assets:checking"), &runs);

        assert!(output.contains("recon-9"));
        assert!(output.contains("recon-9"));
        assert!(output.contains("$0.00"));
        assert!(output.contains("recon-10"));
        assert!(output.contains("-$5.00"));
        assert!(output.contains("1970-01-01 00:28:20 UTC"));
    }
}
