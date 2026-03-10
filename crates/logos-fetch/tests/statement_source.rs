use logos_fetch::{OutputFormat, StatementSource};

#[test]
fn statement_source_prefers_csv_before_pdf_when_configured() {
    let source = StatementSource::new(
        "amex:blue",
        "american-express",
        "assets:amex",
        vec![OutputFormat::Csv, OutputFormat::Pdf],
    )
    .expect("valid source");

    assert_eq!(source.preferred_format(), Some(OutputFormat::Csv));
}

#[test]
fn statement_source_rejects_empty_ids() {
    assert!(
        StatementSource::new(
            "",
            "american-express",
            "assets:amex",
            vec![OutputFormat::Pdf]
        )
        .is_err()
    );
}
