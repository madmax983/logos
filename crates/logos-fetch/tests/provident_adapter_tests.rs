use logos_fetch::{
    FetchRequest, FetchRunStatus, OutputFormat, ProvidentAdapter, SecretBundle, StatementAdapter,
    StatementSource,
};

#[tokio::test]
async fn provident_adapter_normalize_statement_month_yyyy_mm_dd() {
    let source = StatementSource::new(
        "pcu:checking",
        "provident-credit-union",
        "assets:checking",
        vec![OutputFormat::Pdf],
    )
    .expect("source");
    let request = FetchRequest::new(&source, "2026-02").expect("request");
    let secrets = SecretBundle::new("user", "pass", Some("123456")).expect("secrets");

    let runner_output_path =
        std::env::temp_dir().join("provident-normalize-statement-month-YYYY-MM-DD.json");
    std::fs::write(
        &runner_output_path,
        r#"{
  "status": "downloaded",
  "statement_month": "2026-02-28",
  "opening_balance_cents": 100000,
  "closing_balance_cents": 198766,
  "artifact_path": "provident-2026-02.pdf",
  "output_format": "pdf"
}"#,
    )
    .expect("write runner output");

    let adapter = ProvidentAdapter::from_runner_output_path(&runner_output_path).expect("adapter");
    let result = adapter.fetch(&request, &secrets).await.expect("fetch");

    assert_eq!(result.status(), FetchRunStatus::Downloaded);
    let artifact = result.artifact().expect("artifact");
    assert_eq!(artifact.month_key(), "2026-02");

    std::fs::remove_file(&runner_output_path).ok();
}

#[tokio::test]
async fn provident_adapter_normalize_statement_month_invalid_format() {
    let source = StatementSource::new(
        "pcu:checking",
        "provident-credit-union",
        "assets:checking",
        vec![OutputFormat::Pdf],
    )
    .expect("source");
    let request = FetchRequest::new(&source, "2026-02").expect("request");
    let secrets = SecretBundle::new("user", "pass", Some("123456")).expect("secrets");

    let runner_output_path =
        std::env::temp_dir().join("provident-normalize-statement-month-invalid-format.json");
    std::fs::write(
        &runner_output_path,
        r#"{
  "status": "downloaded",
  "statement_month": "2026-02x28",
  "opening_balance_cents": 100000,
  "closing_balance_cents": 198766,
  "artifact_path": "provident-2026-02.pdf",
  "output_format": "pdf"
}"#,
    )
    .expect("write runner output");

    let adapter = ProvidentAdapter::from_runner_output_path(&runner_output_path).expect("adapter");
    let result = adapter.fetch(&request, &secrets).await;

    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().to_string(),
        "provident runner output month '2026-02x28' must be YYYY-MM or YYYY-MM-DD"
    );

    std::fs::remove_file(&runner_output_path).ok();
}

#[tokio::test]
async fn provident_adapter_normalize_statement_month_invalid_month_key() {
    let source = StatementSource::new(
        "pcu:checking",
        "provident-credit-union",
        "assets:checking",
        vec![OutputFormat::Pdf],
    )
    .expect("source");
    let request = FetchRequest::new(&source, "2026-02").expect("request");
    let secrets = SecretBundle::new("user", "pass", Some("123456")).expect("secrets");

    let runner_output_path =
        std::env::temp_dir().join("provident-normalize-statement-month-invalid-month-key.json");
    std::fs::write(
        &runner_output_path,
        r#"{
  "status": "downloaded",
  "statement_month": "202x-02",
  "opening_balance_cents": 100000,
  "closing_balance_cents": 198766,
  "artifact_path": "provident-2026-02.pdf",
  "output_format": "pdf"
}"#,
    )
    .expect("write runner output");

    let adapter = ProvidentAdapter::from_runner_output_path(&runner_output_path).expect("adapter");
    let result = adapter.fetch(&request, &secrets).await;

    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().to_string(),
        "provident runner output month '202x-02' must be YYYY-MM or YYYY-MM-DD"
    );

    std::fs::remove_file(&runner_output_path).ok();
}

#[tokio::test]
async fn provident_adapter_normalize_statement_month_invalid_date_yyyy_mm_dd() {
    let source = StatementSource::new(
        "pcu:checking",
        "provident-credit-union",
        "assets:checking",
        vec![OutputFormat::Pdf],
    )
    .expect("source");
    let request = FetchRequest::new(&source, "2026-02").expect("request");
    let secrets = SecretBundle::new("user", "pass", Some("123456")).expect("secrets");

    let runner_output_path = std::env::temp_dir()
        .join("provident-normalize-statement-month-invalid-date-YYYY-MM-DD.json");
    std::fs::write(
        &runner_output_path,
        r#"{
  "status": "downloaded",
  "statement_month": "202x-02-28",
  "opening_balance_cents": 100000,
  "closing_balance_cents": 198766,
  "artifact_path": "provident-2026-02.pdf",
  "output_format": "pdf"
}"#,
    )
    .expect("write runner output");

    let adapter = ProvidentAdapter::from_runner_output_path(&runner_output_path).expect("adapter");
    let result = adapter.fetch(&request, &secrets).await;

    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().to_string(),
        "provident runner output month '202x-02-28' must be YYYY-MM or YYYY-MM-DD"
    );

    std::fs::remove_file(&runner_output_path).ok();
}
