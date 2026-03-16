#![allow(clippy::inconsistent_digit_grouping)]
use logos_fetch::{FetchRunStatus, FetchedStatementArtifact, OutputFormat};

#[test]
fn fetched_artifact_tracks_balances_and_format() {
    let artifact = FetchedStatementArtifact::new(
        "pcu:checking",
        "assets:checking",
        OutputFormat::Pdf,
        "artifacts/statements/pcu-2026-02.pdf",
        "2026-02",
        100_00,
        250_00,
    )
    .expect("artifact");

    assert_eq!(artifact.month_key(), "2026-02");
    assert_eq!(artifact.closing_balance_cents(), 250_00);
}

#[test]
fn needs_attention_is_not_treated_as_terminal_success() {
    assert!(!FetchRunStatus::NeedsAttention.is_success());
    assert!(FetchRunStatus::Downloaded.is_success());
}
