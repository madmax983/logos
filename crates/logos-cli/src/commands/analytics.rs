use crate::args::CliError;
use logos_app::CliRuntime;

/// Handles `ledger analytics snapshot create`.
///
/// # Errors
///
/// Returns an error when runtime initialization, snapshot creation, or manifest persistence fails.
pub fn snapshot_create(
    as_of_valid_time_us: Option<i64>,
    as_of_tx_time_us: Option<i64>,
    schema_version: i64,
    supersedes_artifact_id: Option<&str>,
) -> Result<(), CliError> {
    let mut runtime = CliRuntime::new().map_err(|err: logos_app::RuntimeError| {
        CliError::CommandRuntimeFailed {
            command: "analytics.snapshot.create".to_owned(),
            message: format!("runtime initialization failed: {err}"),
        }
    })?;
    let manifest = runtime
        .create_analytics_snapshot(
            as_of_valid_time_us,
            as_of_tx_time_us,
            schema_version,
            supersedes_artifact_id,
        )
        .map_err(
            |err: logos_app::RuntimeError| CliError::CommandRuntimeFailed {
                command: "analytics.snapshot.create".to_owned(),
                message: err.to_string(),
            },
        )?;

    println!(
        "{}",
        render_snapshot_manifest("analytics.snapshot.create", &manifest)
    );
    Ok(())
}

/// Handles `ledger analytics snapshot list`.
///
/// # Errors
///
/// Returns an error when runtime initialization fails.
pub fn snapshot_list() -> Result<(), CliError> {
    let runtime = CliRuntime::new().map_err(|err: logos_app::RuntimeError| {
        CliError::CommandRuntimeFailed {
            command: "analytics.snapshot.list".to_owned(),
            message: format!("runtime initialization failed: {err}"),
        }
    })?;
    let manifests = runtime.list_analytics_snapshots();
    if manifests.is_empty() {
        println!("analytics.snapshot.list empty=true count=0");
        return Ok(());
    }

    println!(
        "analytics.snapshot.list empty=false count={}",
        manifests.len()
    );
    for manifest in manifests {
        println!(
            "{}",
            render_snapshot_manifest("analytics.snapshot.item", &manifest)
        );
    }
    Ok(())
}

/// Handles `ledger analytics snapshot show`.
///
/// # Errors
///
/// Returns an error when runtime initialization fails or the manifest id is missing.
pub fn snapshot_show(artifact_id: &str) -> Result<(), CliError> {
    let runtime = CliRuntime::new().map_err(|err: logos_app::RuntimeError| {
        CliError::CommandRuntimeFailed {
            command: "analytics.snapshot.show".to_owned(),
            message: format!("runtime initialization failed: {err}"),
        }
    })?;
    let Some(manifest) = runtime.get_analytics_snapshot(artifact_id) else {
        return Err(CliError::CommandRuntimeFailed {
            command: "analytics.snapshot.show".to_owned(),
            message: format!("analytics artifact '{artifact_id}' not found"),
        });
    };

    println!(
        "{}",
        render_snapshot_manifest("analytics.snapshot.show", &manifest)
    );
    Ok(())
}

fn render_snapshot_manifest(
    prefix: &str,
    manifest: &logos_store_aletheia::model::StoredAnalyticsArtifactManifest,
) -> String {
    let supersedes = manifest.supersedes_artifact_id().unwrap_or("");
    format!(
        "{prefix} artifact_id={} kind={} schema_version={} row_count={} hash={} uri={} valid_us={} tx_us={} created_us={} supersedes={}",
        manifest.artifact_id(),
        manifest.artifact_kind(),
        manifest.schema_version(),
        manifest.row_count(),
        manifest.content_hash(),
        manifest.artifact_uri(),
        manifest.snapshot_valid_at().wallclock(),
        manifest.snapshot_tx_at().wallclock(),
        manifest.created_at().wallclock(),
        supersedes
    )
}

#[cfg(test)]
mod tests {
    use super::render_snapshot_manifest;
    use logos_store_aletheia::model::StoredAnalyticsArtifactManifest;

    #[test]
    fn render_snapshot_manifest_is_deterministic() {
        let manifest = StoredAnalyticsArtifactManifest::new(
            "artifact-7",
            "parquet",
            "C:\\artifacts\\abc.parquet",
            "deadbeef",
            2,
            19,
            1_700_000_000_i64.into(),
            1_700_000_001_i64.into(),
            1_700_000_002_i64.into(),
            Some("artifact-6"),
            "valid:1700000000000000|tx:1700000001000000",
        );
        let output = render_snapshot_manifest("analytics.snapshot.show", &manifest);
        assert_eq!(
            output,
            "analytics.snapshot.show artifact_id=artifact-7 kind=parquet schema_version=2 row_count=19 hash=deadbeef uri=C:\\artifacts\\abc.parquet valid_us=1700000000 tx_us=1700000001 created_us=1700000002 supersedes=artifact-6"
        );
    }
}
