use crate::args::CliError;
use logos_app::CliRuntime;
use logos_store_aletheia::model::StoredMonthClose;

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
    let mut runtime = CliRuntime::new().map_err(|err: logos_app::RuntimeError| {
        CliError::CommandRuntimeFailed {
            command: "close.month".to_owned(),
            message: format!("runtime initialization failed: {err}"),
        }
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
        .map_err(
            |err: logos_app::RuntimeError| CliError::CommandRuntimeFailed {
                command: "close.month".to_owned(),
                message: err.to_string(),
            },
        )?;

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
    format!(
        "close.month close_id={} month={} checking_account={} reconciliation_run_id={} analytics_artifact_id={} closed_at_us={}",
        close.close_id(),
        close.month_key(),
        close.checking_account(),
        close.reconciliation_run_id(),
        analytics_artifact_id.unwrap_or(""),
        close.closed_at().wallclock()
    )
}

#[cfg(test)]
mod tests {
    use super::render_close_month_output;
    use logos_store_aletheia::model::StoredMonthClose;

    #[test]
    fn render_close_month_output_is_deterministic() {
        let close = StoredMonthClose::new(
            "close-3",
            "2026-03",
            "assets:checking",
            "recon-11",
            Some("artifact-7"),
            1_700_000_555_i64.into(),
        );

        let output = render_close_month_output(&close, close.analytics_artifact_id());
        assert_eq!(
            output,
            "close.month close_id=close-3 month=2026-03 checking_account=assets:checking reconciliation_run_id=recon-11 analytics_artifact_id=artifact-7 closed_at_us=1700000555"
        );
    }
}
