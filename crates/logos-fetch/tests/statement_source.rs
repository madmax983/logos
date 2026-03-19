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

#[test]
fn statement_source_accessors() {
    use logos_fetch::{OutputFormat, StatementSource};
    let source = StatementSource::new(
        "pcu:checking",
        "provident-credit-union",
        "assets:checking",
        vec![OutputFormat::Pdf, OutputFormat::Csv],
    )
    .expect("source");

    assert_eq!(source.institution_id(), "provident-credit-union");
    assert_eq!(
        source.format_preference(),
        &[OutputFormat::Pdf, OutputFormat::Csv]
    );
}
