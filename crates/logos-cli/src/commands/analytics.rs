use crate::args::CliError;
use comfy_table::{Cell, Color};
use logos_core::format::us_timestamp;

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
    let mut runtime =
        crate::runtime::init_runtime().map_err(|err| CliError::CommandRuntimeFailed {
            command: "analytics.snapshot.create".to_owned(),
            message: format!("{err}"),
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

    println!("{}", render_snapshot_manifest(&manifest));
    Ok(())
}

/// Handles `ledger analytics snapshot list`.
///
/// # Errors
///
/// Returns an error when runtime initialization fails.
pub fn snapshot_list() -> Result<(), CliError> {
    let runtime = crate::runtime::init_runtime().map_err(|err| CliError::CommandRuntimeFailed {
        command: "analytics.snapshot.list".to_owned(),
        message: format!("{err}"),
    })?;
    let manifests = runtime.list_analytics_snapshots();
    println!("{}", render_snapshot_manifest_list(&manifests));
    Ok(())
}

fn render_snapshot_manifest_list(
    manifests: &[logos_store::StoredAnalyticsArtifactManifest],
) -> String {
    if manifests.is_empty() {
        return "No analytics snapshots found. Try creating one with 'ledger analytics snapshot create'.".to_owned();
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
        table.add_row(manifest_row(manifest));
    }

    table.to_string()
}

/// Handles `ledger analytics snapshot show`.
///
/// # Errors
///
/// Returns an error when runtime initialization fails or the manifest id is missing.
pub fn snapshot_show(artifact_id: &str) -> Result<(), CliError> {
    let runtime = crate::runtime::init_runtime().map_err(|err| CliError::CommandRuntimeFailed {
        command: "analytics.snapshot.show".to_owned(),
        message: format!("{err}"),
    })?;
    let Some(manifest) = runtime.get_analytics_snapshot(artifact_id) else {
        return Err(CliError::CommandRuntimeFailed {
            command: "analytics.snapshot.show".to_owned(),
            message: format!("analytics artifact '{artifact_id}' not found"),
        });
    };

    println!("{}", render_snapshot_manifest(&manifest));
    Ok(())
}

/// Handles `ledger analytics sankey`.
///
/// # Errors
///
/// Returns an error when runtime initialization fails or querying transactions fails.
pub fn sankey() -> Result<(), CliError> {
    use chrono::Utc;
    use logos_core::mermaid_exporter::MermaidSankeyExporter;

    let runtime = crate::runtime::init_runtime().map_err(|err| CliError::CommandRuntimeFailed {
        command: "analytics.sankey".to_owned(),
        message: format!("{err}"),
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
    use crossterm::style::Stylize;

    let header = "📊 Sankey Flow Diagram Generated!".green().bold();
    let instruction =
        "Copy the code below and paste it into https://mermaid.live to view your cashflow:"
            .italic();

    format!("\n{header}\n{instruction}\n\n{raw_mermaid}\n")
}

fn render_snapshot_manifest(manifest: &logos_store::StoredAnalyticsArtifactManifest) -> String {
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

    table.add_row(manifest_row(manifest));

    table.to_string()
}

fn manifest_row(manifest: &logos_store::StoredAnalyticsArtifactManifest) -> Vec<Cell> {
    let supersedes = manifest.supersedes_artifact_id().unwrap_or("");
    vec![
        Cell::new(manifest.artifact_id()).fg(Color::Blue),
        Cell::new(manifest.artifact_kind()).fg(Color::Green),
        Cell::new(manifest.schema_version().to_string()),
        Cell::new(manifest.row_count().to_string()),
        Cell::new(manifest.content_hash()),
        Cell::new(manifest.artifact_uri()),
        Cell::new(us_timestamp(manifest.snapshot_valid_at())),
        Cell::new(us_timestamp(manifest.snapshot_tx_at())),
        Cell::new(us_timestamp(manifest.created_at())),
        Cell::new(supersedes),
    ]
}

#[cfg(test)]
mod tests {
    use super::{render_sankey_output, render_snapshot_manifest, render_snapshot_manifest_list};
    use logos_store::StoredAnalyticsArtifactManifest;

    #[test]
    fn render_sankey_output_is_deterministic() {
        use crossterm::style::Stylize;

        let raw = "```mermaid\nsankey-beta\nincome:salary,assets:checking,500.00\n```\n";
        let output = render_sankey_output(raw);

        let header = "📊 Sankey Flow Diagram Generated!".green().bold();
        let instruction =
            "Copy the code below and paste it into https://mermaid.live to view your cashflow:"
                .italic();

        let expected = format!("\n{header}\n{instruction}\n\n{raw}\n");
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
            1_800_000_000_i64,
            1_800_000_001_i64,
            1_800_000_002_i64,
            Some("artifact-7"),
            "valid:1800000000000000|tx:1800000001000000",
        )];
        let output = render_snapshot_manifest_list(&manifests);
        assert!(output.contains("artifact-8"));
        assert!(output.contains("parquet"));
        assert!(output.contains("cafebabe"));
        assert!(output.contains("C:\\artifacts\\def.parquet"));
        assert!(output.contains("artifact-7"));
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
            1_700_000_000_i64,
            1_700_000_001_i64,
            1_700_000_002_i64,
            Some("artifact-6"),
            "valid:1700000000000000|tx:1700000001000000",
        );
        let output = render_snapshot_manifest(&manifest);
        assert!(output.contains("artifact-7"));
        assert!(output.contains("parquet"));
        assert!(output.contains("deadbeef"));
        assert!(output.contains("C:\\artifacts\\abc.parquet"));
        assert!(output.contains("artifact-6"));
    }
}

/// Handles `ledger analytics net-worth`.
///
/// # Errors
///
/// Returns an error when runtime initialization fails.
#[allow(clippy::unnecessary_wraps)]
pub fn net_worth_project(
    initial_net_worth_cents: i64,
    monthly_savings_cents: i64,
    months: u16,
) -> Result<(), CliError> {
    use logos_core::net_worth_projector::NetWorthProjector;

    let projector = NetWorthProjector::new(initial_net_worth_cents, monthly_savings_cents);
    let (timeline, _) = projector.project_timeline(months);

    let mut table = comfy_table::Table::new();
    table.load_preset(comfy_table::presets::UTF8_FULL);
    table.set_header(vec!["Month", "Net Worth", "Saved Cash", "Vested Value"]);

    for month in timeline {
        table.add_row(vec![
            comfy_table::Cell::new(month.month_index.to_string()),
            comfy_table::Cell::new(logos_core::format::currency(month.net_worth_cents))
                .fg(comfy_table::Color::Green),
            comfy_table::Cell::new(logos_core::format::currency(month.saved_cents)),
            comfy_table::Cell::new(logos_core::format::currency(month.vested_value_cents)),
        ]);
    }

    println!(
        "analytics.net-worth
{table}"
    );

    Ok(())
}



#[cfg(test)]
mod net_worth_tests {


    #[test]
    fn test_net_worth_project_calculates_correctly() {
        let result = super::net_worth_project(10_000_000, 500_000, 12);
        assert!(result.is_ok());
    }
}
