use crate::args::CliError;
use comfy_table::{Cell, Color};
use logos_core::format::us_timestamp;

use logos_store::{StoredFetchRun, StoredFetchRunStatus};

/// Handles `ledger fetch list-runs`.
///
/// # Errors
///
/// Returns an error when runtime initialization fails.
pub fn list_runs(month_key: Option<&str>, checking_account: Option<&str>) -> Result<(), CliError> {
    let runtime = crate::runtime::init_runtime().map_err(|err| CliError::CommandRuntimeFailed {
        command: "fetch.list".to_owned(),
        message: format!("{err}"),
    })?;
    let runs = runtime.list_fetch_runs(month_key, checking_account);
    println!("{}", render_list_output(month_key, checking_account, &runs));
    Ok(())
}

/// Handles `ledger fetch show-run`.
///
/// # Errors
///
/// Returns an error when runtime initialization fails.
pub fn show_run(run_id: &str) -> Result<(), CliError> {
    let runtime = crate::runtime::init_runtime().map_err(|err| CliError::CommandRuntimeFailed {
        command: "fetch.show".to_owned(),
        message: format!("{err}"),
    })?;
    let Some(run) = runtime.fetch_run(run_id) else {
        return Err(CliError::CommandRuntimeFailed {
            command: "fetch.show".to_owned(),
            message: format!("fetch run '{run_id}' not found"),
        });
    };
    println!("{}", render_show_output(&run));
    Ok(())
}

fn status_cell(status: StoredFetchRunStatus) -> Cell {
    let text = status.as_str();
    match status {
        StoredFetchRunStatus::Downloaded
        | StoredFetchRunStatus::Imported
        | StoredFetchRunStatus::NoNewStatement => Cell::new(text).fg(Color::Green),
        StoredFetchRunStatus::NeedsAttention => Cell::new(text).fg(Color::Yellow),
        StoredFetchRunStatus::Failed => Cell::new(text).fg(Color::Red),
    }
}

fn render_list_output(
    month_key: Option<&str>,
    checking_account: Option<&str>,
    runs: &[StoredFetchRun],
) -> String {
    let filter_month = month_key.unwrap_or("*");
    let filter_account = checking_account.unwrap_or("*");
    if runs.is_empty() {
        return format!(
            "fetch.list filter_month={filter_month} filter_checking_account={filter_account} count=0"
        );
    }

    let mut table = comfy_table::Table::new();
    table.load_preset(comfy_table::presets::UTF8_FULL);
    table.set_header(vec![
        "Run ID",
        "Month",
        "Account",
        "Source",
        "Status",
        "Created At",
    ]);

    for run in runs {
        table.add_row(vec![
            Cell::new(run.run_id()),
            Cell::new(run.month_key()),
            Cell::new(run.ledger_account()),
            Cell::new(run.source_id()),
            status_cell(run.status()),
            Cell::new(us_timestamp(run.created_at())),
        ]);
    }

    format!(
        "fetch.list filter_month={filter_month} filter_checking_account={filter_account} count={}\n{table}",
        runs.len()
    )
}

fn render_show_output(run: &StoredFetchRun) -> String {
    let mut table = comfy_table::Table::new();
    table.load_preset(comfy_table::presets::UTF8_FULL);
    table.set_header(vec![
        "Run ID",
        "Source",
        "Institution",
        "Account",
        "Month",
        "Status",
        "Artifact",
        "Opening",
        "Closing",
        "Error",
        "Created At",
    ]);
    table.add_row(vec![
        Cell::new(run.run_id()),
        Cell::new(run.source_id()),
        Cell::new(run.institution_id()),
        Cell::new(run.ledger_account()),
        Cell::new(run.month_key()),
        status_cell(run.status()),
        Cell::new(run.artifact_path().unwrap_or("-")),
        run.opening_balance_cents().map_or_else(
            || Cell::new("-"),
            |value| Cell::new(logos_core::format::currency(value)).set_alignment(comfy_table::CellAlignment::Right).fg(Color::Blue),
        ),
        run.closing_balance_cents().map_or_else(
            || Cell::new("-"),
            |value| Cell::new(logos_core::format::currency(value)).set_alignment(comfy_table::CellAlignment::Right).fg(Color::Blue),
        ),
        run.error_summary()
            .map_or_else(|| Cell::new("-"), |err| Cell::new(err).fg(Color::Red)),
        Cell::new(us_timestamp(run.created_at())),
    ]);
    table.to_string()
}

#[cfg(test)]
mod tests {
    use super::{render_list_output, render_show_output};
    use logos_store::{StoredFetchArtifactFormat, StoredFetchRun, StoredFetchRunStatus};

    #[test]
    fn render_list_output_is_deterministic() {
        let run = StoredFetchRun::new(
            "fetch-3",
            "pcu:checking",
            "provident-credit-union",
            "assets:checking",
            "2026-03",
            StoredFetchRunStatus::Downloaded,
            Some("C:\\statements\\pcu-2026-03.pdf"),
            Some(StoredFetchArtifactFormat::Pdf),
            Some(100_000),
            Some(198_766),
            None,
            1_700_000_333_i64,
        );
        let output = render_list_output(Some("2026-03"), Some("assets:checking"), &[run]);

        assert!(output.contains("fetch.list filter_month=2026-03"));
        assert!(output.contains("fetch-3"));
        assert!(output.contains("pcu:checking"));
        assert!(output.contains("downloaded"));
        assert!(output.contains("1970-01-01 00:28:20 UTC"));
    }

    #[test]
    fn render_show_output_with_balances_is_deterministic() {
        let run = StoredFetchRun::new(
            "fetch-3",
            "pcu:checking",
            "provident-credit-union",
            "assets:checking",
            "2026-03",
            StoredFetchRunStatus::Downloaded,
            Some("C:\\statements\\pcu-2026-03.pdf"),
            Some(StoredFetchArtifactFormat::Pdf),
            Some(100_000),
            Some(198_766),
            None,
            1_700_000_333_i64,
        );
        let output = render_show_output(&run);

        assert!(output.contains("fetch-3"));
        assert!(output.contains("provident-credit-union"));
        assert!(output.contains("downloaded"));
        assert!(output.contains("$1,000.00"));
        assert!(output.contains("$1,987.66"));
        assert!(output.contains("1970-01-01 00:28:20 UTC"));
    }

    #[test]
    fn render_show_output_is_deterministic() {
        let run = StoredFetchRun::new(
            "fetch-4",
            "amex:blue",
            "american-express",
            "liabilities:amex",
            "2026-04",
            StoredFetchRunStatus::NeedsAttention,
            None,
            None,
            None,
            None,
            Some("mfa challenge required"),
            1_700_000_444_i64,
        );
        let output = render_show_output(&run);

        assert!(output.contains("fetch-4"));
        assert!(output.contains("american-express"));
        assert!(output.contains("needs_attention"));
        assert!(output.contains("mfa challenge required"));
        assert!(output.contains("1970-01-01 00:28:20 UTC"));
    }
}
