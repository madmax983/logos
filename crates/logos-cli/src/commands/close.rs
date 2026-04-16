use crate::args::CliError;
use crate::format::us_timestamp;

use logos_store::model::StoredMonthClose;

/// Handles `ledger close month`.
///
/// # Errors
///
/// Returns an error when runtime initialization or close persistence fails.
pub fn month(
    month_key: Option<&str>,
    checking_account: &str,
    run_id: &str,
    analytics_artifact_id: Option<&str>,
) -> Result<(), CliError> {
    let mut runtime = crate::runtime::init_runtime().map_err(|err| CliError::CommandRuntimeFailed {
        command: "close.month".to_owned(),
        message: format!("{err}"),
    })?;
    let resolved_month_key = if let Some(explicit_month) = month_key {
        explicit_month.to_owned()
    } else {
        let run =
            runtime
                .reconciliation_run(run_id)
                .ok_or_else(|| CliError::CommandRuntimeFailed {
                    command: "close.month".to_owned(),
                    message: format!("reconciliation run '{run_id}' not found"),
                })?;
        run.month_key().to_owned()
    };

    let close = runtime
        .close_month(
            &resolved_month_key,
            checking_account,
            run_id,
            analytics_artifact_id,
        )
        .map_err(|err| CliError::CommandRuntimeFailed {
            command: "close.month".to_owned(),
            message: err.to_string(),
        })?;

    println!(
        "{}",
        render_close_month_output(
            &close,
            analytics_artifact_id.or_else(|| close.analytics_artifact_id())
        )
    );
    Ok(())
}

fn render_close_month_output(
    close: &StoredMonthClose,
    analytics_artifact_id: Option<&str>,
) -> String {
    let mut table = comfy_table::Table::new();
    table.load_preset(comfy_table::presets::UTF8_FULL);
    table.set_header(vec![
        "Close ID",
        "Month",
        "Account",
        "Recon Run ID",
        "Analytics ID",
        "Closed At",
    ]);

    table.add_row(vec![
        close.close_id().to_owned(),
        close.month_key().to_owned(),
        close.checking_account().to_owned(),
        close.reconciliation_run_id().to_owned(),
        analytics_artifact_id.unwrap_or("").to_owned(),
        us_timestamp(close.closed_at()),
    ]);

    format!("close.month\n{table}")
}

#[cfg(test)]
mod tests {
    use super::render_close_month_output;
    use logos_store::model::StoredMonthClose;

    #[test]
    fn render_close_month_output_is_deterministic() {
        let close = StoredMonthClose::new(
            "close-3",
            "2026-03",
            "assets:checking",
            "recon-11",
            Some("artifact-7"),
            1_700_000_555_i64,
        );

        let output = render_close_month_output(&close, close.analytics_artifact_id());
        assert!(output.contains("close.month"));
        assert!(output.contains("close-3"));
        assert!(output.contains("2026-03"));
        assert!(output.contains("assets:checking"));
        assert!(output.contains("recon-11"));
        assert!(output.contains("artifact-7"));
        assert!(output.contains("1970-01-01 00:28:20 UTC"));
    }
}
