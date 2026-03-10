use std::path::Path;

use logos_fetch::{
    FakeStatementAdapter, FetchRequest, FetchRunStatus, OutputFormat, SecretBundle,
    StatementAdapter, StatementSource,
};

#[tokio::test]
async fn fake_adapter_returns_downloaded_artifact_for_happy_path() {
    let source = StatementSource::new(
        "pcu:checking",
        "provident-credit-union",
        "assets:checking",
        vec![OutputFormat::Pdf],
    )
    .expect("source");
    let request = FetchRequest::new(&source, "2026-02").expect("request");
    let secrets = SecretBundle::new("user", "pass", Some("123456")).expect("secrets");

    let result = FakeStatementAdapter::download_fixture_statement()
        .fetch(&request, &secrets)
        .await
        .expect("fetch");

    assert_eq!(result.status(), FetchRunStatus::Downloaded);
    let artifact = result.artifact().expect("artifact");
    assert!(Path::new(artifact.artifact_path()).exists());
    assert_eq!(artifact.opening_balance_cents(), 100_000);
    assert_eq!(artifact.closing_balance_cents(), 198_766);
}

#[tokio::test]
async fn fake_adapter_rejects_months_without_a_matching_fixture() {
    let source = StatementSource::new(
        "pcu:checking",
        "fake-fixture",
        "assets:checking",
        vec![OutputFormat::Pdf],
    )
    .expect("source");
    let request = FetchRequest::new(&source, "2026-03").expect("request");
    let secrets = SecretBundle::new("user", "pass", Some("123456")).expect("secrets");

    let err = FakeStatementAdapter::download_fixture_statement()
        .fetch(&request, &secrets)
        .await
        .expect_err("fixture month mismatch should reject");

    assert!(
        err.to_string()
            .contains("fake fixture only supports statement month 2026-02"),
        "unexpected error: {err}"
    );
}

#[tokio::test]
async fn fake_adapter_can_return_needs_attention_without_an_artifact() {
    let source = StatementSource::new(
        "m1:taxable",
        "fake-needs-attention",
        "assets:checking",
        vec![OutputFormat::Pdf],
    )
    .expect("source");
    let request = FetchRequest::new(&source, "2026-02").expect("request");
    let secrets = SecretBundle::new("user", "pass", Some("123456")).expect("secrets");

    let result = FakeStatementAdapter::needs_attention("mfa challenge required")
        .fetch(&request, &secrets)
        .await
        .expect("fetch");

    assert_eq!(result.status(), FetchRunStatus::NeedsAttention);
    assert_eq!(result.error_summary(), Some("mfa challenge required"));
    assert!(result.artifact().is_none());
}
