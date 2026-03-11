use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use logos_fetch::{
    FetchRequest, FetchRunStatus, OutputFormat, ProvidentAdapter, SecretBundle, StatementAdapter,
    StatementSource,
};

fn temp_path(prefix: &str, extension: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!("logos-fetch-{prefix}-{nanos}.{extension}"))
}

fn cleanup_file(path: &Path) {
    if path.exists() {
        let _ = std::fs::remove_file(path);
    }
}

#[tokio::test]
async fn provident_adapter_maps_downloaded_statement_into_metadata() {
    let source = StatementSource::new(
        "pcu:checking",
        "provident-credit-union",
        "assets:checking",
        vec![OutputFormat::Pdf],
    )
    .expect("source");
    let request = FetchRequest::new(&source, "2026-02").expect("request");
    let secrets = SecretBundle::new("user", "pass", Some("123456")).expect("secrets");
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("provident_statement_metadata.json");

    let result = ProvidentAdapter::from_runner_output_path(&fixture_path)
        .expect("adapter")
        .fetch(&request, &secrets)
        .await
        .expect("fetch");

    assert_eq!(result.status(), FetchRunStatus::Downloaded);
    assert_eq!(result.error_summary(), None);
    let artifact = result.artifact().expect("artifact");
    assert_eq!(artifact.source_id(), "pcu:checking");
    assert_eq!(artifact.ledger_account(), "assets:checking");
    assert_eq!(artifact.output_format(), OutputFormat::Pdf);
    assert_eq!(artifact.month_key(), "2026-02");
    assert_eq!(artifact.opening_balance_cents(), 100_000);
    assert_eq!(artifact.closing_balance_cents(), 198_766);
    assert!(artifact.artifact_path().ends_with("provident-2026-02.pdf"));
    assert!(Path::new(artifact.artifact_path()).exists());
}

#[tokio::test]
async fn provident_adapter_maps_needs_attention_runner_state_into_fetch_result() {
    let source = StatementSource::new(
        "pcu:checking",
        "provident-credit-union",
        "assets:checking",
        vec![OutputFormat::Pdf],
    )
    .expect("source");
    let request = FetchRequest::new(&source, "2026-02").expect("request");
    let secrets = SecretBundle::new("user", "pass", Some("123456")).expect("secrets");
    let runner_output_path = temp_path("provident-needs-attention", "json");
    std::fs::write(
        &runner_output_path,
        r#"{
  "status": "needs_attention",
  "error_summary": "mfa challenge required"
}"#,
    )
    .expect("write runner output");

    let result = ProvidentAdapter::from_runner_output_path(&runner_output_path)
        .expect("adapter")
        .fetch(&request, &secrets)
        .await
        .expect("fetch");

    assert_eq!(result.status(), FetchRunStatus::NeedsAttention);
    assert_eq!(result.error_summary(), Some("mfa challenge required"));
    assert!(result.artifact().is_none());

    cleanup_file(&runner_output_path);
}
