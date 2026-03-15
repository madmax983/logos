use crate::{args::CliError, runtime::CliRuntime};

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
    let mut runtime = CliRuntime::new().map_err(|err| CliError::CommandRuntimeFailed {
        command: "analytics.snapshot.create".to_owned(),
        message: format!("runtime initialization failed: {err}"),
    })?;
    let manifest = runtime
        .create_analytics_snapshot(
            as_of_valid_time_us,
            as_of_tx_time_us,
            schema_version,
            supersedes_artifact_id,
        )
        .map_err(|err| CliError::CommandRuntimeFailed {
            command: "analytics.snapshot.create".to_owned(),
            message: err.to_string(),
        })?;

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
    let runtime = CliRuntime::new().map_err(|err| CliError::CommandRuntimeFailed {
        command: "analytics.snapshot.list".to_owned(),
        message: format!("runtime initialization failed: {err}"),
    })?;
    let manifests = runtime.list_analytics_snapshots();
    println!("{}", render_snapshot_manifest_list(&manifests));
    Ok(())
}

fn render_snapshot_manifest_list(
    manifests: &[logos_store_aletheia::model::StoredAnalyticsArtifactManifest],
) -> String {
    if manifests.is_empty() {
        return "analytics.snapshot.list empty=true count=0".to_owned();
    }

    let mut table = comfy_table::Table::new();
    table.load_preset(comfy_table::presets::UTF8_FULL);
    table.set_header(vec![
        "Artifact ID",
        "Kind",
        "Schema",
        "Rows",
        "Hash",
        "URI",
        "Valid US",
        "Tx US",
        "Created US",
        "Supersedes",
    ]);

    for manifest in manifests {
        let supersedes = manifest.supersedes_artifact_id().unwrap_or("");
        table.add_row(vec![
            manifest.artifact_id().to_owned(),
            manifest.artifact_kind().to_owned(),
            manifest.schema_version().to_string(),
            manifest.row_count().to_string(),
            manifest.content_hash().to_owned(),
            manifest.artifact_uri().to_owned(),
            manifest.snapshot_valid_at().wallclock().to_string(),
            manifest.snapshot_tx_at().wallclock().to_string(),
            manifest.created_at().wallclock().to_string(),
            supersedes.to_owned(),
        ]);
    }

    format!(
        "analytics.snapshot.list empty=false count={}\n{table}",
        manifests.len()
    )
}

/// Handles `ledger analytics snapshot show`.
///
/// # Errors
///
/// Returns an error when runtime initialization fails or the manifest id is missing.
pub fn snapshot_show(artifact_id: &str) -> Result<(), CliError> {
    let runtime = CliRuntime::new().map_err(|err| CliError::CommandRuntimeFailed {
        command: "analytics.snapshot.show".to_owned(),
        message: format!("runtime initialization failed: {err}"),
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

/// Handles `ledger analytics sankey`.
///
/// # Errors
///
/// Returns an error when runtime initialization fails or querying transactions fails.
pub fn sankey() -> Result<(), CliError> {
    use chrono::Utc;
    use logos_core::experimental::mermaid_exporter::MermaidSankeyExporter;

    let runtime = CliRuntime::new().map_err(|err| CliError::CommandRuntimeFailed {
        command: "analytics.sankey".to_owned(),
        message: format!("runtime initialization failed: {err}"),
    })?;

    let now_us = Utc::now().timestamp_micros();
    let transactions = runtime
        .transactions_as_of_us(now_us, now_us)
        .map_err(|err| CliError::CommandRuntimeFailed {
            command: "analytics.sankey".to_owned(),
            message: format!("failed to retrieve transactions: {err}"),
        })?;

    let mut exporter = MermaidSankeyExporter::new();
    for stored_tx in transactions {
        exporter.add_transaction(stored_tx.transaction().clone());
    }

    println!("{}", render_sankey_output(&exporter.export_sankey()));
    Ok(())
}

fn render_sankey_output(raw_mermaid: &str) -> String {
    format!("analytics.sankey\n{raw_mermaid}")
}

fn render_snapshot_manifest(
    prefix: &str,
    manifest: &logos_store_aletheia::model::StoredAnalyticsArtifactManifest,
) -> String {
    let mut table = comfy_table::Table::new();
    table.load_preset(comfy_table::presets::UTF8_FULL);
    table.set_header(vec![
        "Artifact ID",
        "Kind",
        "Schema",
        "Rows",
        "Hash",
        "URI",
        "Valid US",
        "Tx US",
        "Created US",
        "Supersedes",
    ]);

    let supersedes = manifest.supersedes_artifact_id().unwrap_or("");
    table.add_row(vec![
        manifest.artifact_id().to_owned(),
        manifest.artifact_kind().to_owned(),
        manifest.schema_version().to_string(),
        manifest.row_count().to_string(),
        manifest.content_hash().to_owned(),
        manifest.artifact_uri().to_owned(),
        manifest.snapshot_valid_at().wallclock().to_string(),
        manifest.snapshot_tx_at().wallclock().to_string(),
        manifest.created_at().wallclock().to_string(),
        supersedes.to_owned(),
    ]);

    format!("{prefix}\n{table}")
}

#[cfg(test)]
mod tests {
    use super::{render_sankey_output, render_snapshot_manifest, render_snapshot_manifest_list};
    use logos_store_aletheia::model::StoredAnalyticsArtifactManifest;

    #[test]
    fn render_sankey_output_is_deterministic() {
        let raw = "```mermaid\nsankey-beta\nincome:salary,assets:checking,500.00\n```\n";
        let output = render_sankey_output(raw);
        let expected = "analytics.sankey\n```mermaid\nsankey-beta\nincome:salary,assets:checking,500.00\n```\n";
        assert_eq!(output, expected);
    }

    #[test]
    fn render_snapshot_manifest_list_is_deterministic() {
        let manifests = vec![StoredAnalyticsArtifactManifest::new(
            "artifact-8",
            "parquet",
            "C:\\artifacts\\def.parquet",
            "cafebabe",
            3,
            21,
            1_800_000_000_i64.into(),
            1_800_000_001_i64.into(),
            1_800_000_002_i64.into(),
            Some("artifact-7"),
            "valid:1800000000000000|tx:1800000001000000",
        )];
        let output = render_snapshot_manifest_list(&manifests);
        let expected = "analytics.snapshot.list empty=false count=1\n┌─────────────┬─────────┬────────┬──────┬──────────┬──────────────────────────┬────────────┬────────────┬────────────┬────────────┐
│ Artifact ID ┆ Kind    ┆ Schema ┆ Rows ┆ Hash     ┆ URI                      ┆ Valid US   ┆ Tx US      ┆ Created US ┆ Supersedes │
╞═════════════╪═════════╪════════╪══════╪══════════╪══════════════════════════╪════════════╪════════════╪════════════╪════════════╡
│ artifact-8  ┆ parquet ┆ 3      ┆ 21   ┆ cafebabe ┆ C:\\artifacts\\def.parquet ┆ 1800000000 ┆ 1800000001 ┆ 1800000002 ┆ artifact-7 │
└─────────────┴─────────┴────────┴──────┴──────────┴──────────────────────────┴────────────┴────────────┴────────────┴────────────┘";
        assert_eq!(output, expected);
    }

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
        let expected = "analytics.snapshot.show\n┌─────────────┬─────────┬────────┬──────┬──────────┬──────────────────────────┬────────────┬────────────┬────────────┬────────────┐
│ Artifact ID ┆ Kind    ┆ Schema ┆ Rows ┆ Hash     ┆ URI                      ┆ Valid US   ┆ Tx US      ┆ Created US ┆ Supersedes │
╞═════════════╪═════════╪════════╪══════╪══════════╪══════════════════════════╪════════════╪════════════╪════════════╪════════════╡
│ artifact-7  ┆ parquet ┆ 2      ┆ 19   ┆ deadbeef ┆ C:\\artifacts\\abc.parquet ┆ 1700000000 ┆ 1700000001 ┆ 1700000002 ┆ artifact-6 │
└─────────────┴─────────┴────────┴──────┴──────────┴──────────────────────────┴────────────┴────────────┴────────────┴────────────┘";
        assert_eq!(output, expected);
    }
}
