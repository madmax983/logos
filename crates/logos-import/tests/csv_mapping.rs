use logos_import::{CsvMapping, ImportError, parse_simple_csv_row};

#[test]
fn csv_row_is_mapped_into_import_record() {
    let mapping = CsvMapping {
        timestamp_idx: 0,
        amount_idx: 1,
        memo_idx: 2,
        account_idx: 3,
        category_idx: 4,
        source_id: "test_source".to_string(),
    };
    let line = "2026-02-01T09:30:00,12345,RSU sale,assets:checking,income:rsu";

    let record = parse_simple_csv_row(line, &mapping).expect("parsed");

    assert_eq!(record.timestamp(), "2026-02-01T09:30:00");
    assert_eq!(record.amount_cents(), 12_345);
    assert_eq!(record.memo(), "RSU sale");
    assert_eq!(record.account(), "assets:checking");
    assert_eq!(record.category(), "income:rsu");
}

#[test]
fn csv_row_maps_source_id_and_bounds_checking() {
    let mapping = CsvMapping {
        timestamp_idx: 0,
        amount_idx: 1,
        memo_idx: 2,
        account_idx: 3,
        category_idx: 4,
        source_id: "test".to_string(),
    };
    let line = "2026-02-01T09:30:00,12345,RSU sale,assets:checking,income:rsu";

    let record = parse_simple_csv_row(line, &mapping).expect("parsed");

    // Explicitly test the source id doesn't match an arbitrary string
    assert_ne!(record.source_id(), "");
    assert_ne!(record.source_id(), "xyzzy");

    assert_eq!(record.source_id(), "test");

    // Now pass something with fewer columns, it should error
    let short_line = "2026-02-01T09:30:00,12345,RSU sale,assets:checking";
    assert!(parse_simple_csv_row(short_line, &mapping).is_err());
}

#[test]
fn csv_row_with_quoted_comma_memo_is_parsed_correctly() {
    let mapping = CsvMapping {
        timestamp_idx: 0,
        amount_idx: 1,
        memo_idx: 2,
        account_idx: 3,
        category_idx: 4,
        source_id: "test_source".to_string(),
    };
    let line = "2026-02-01T09:30:00,12345,\"RSU sale, vested\",assets:checking,income:rsu";

    let record = parse_simple_csv_row(line, &mapping).expect("parsed");

    assert_eq!(record.memo(), "RSU sale, vested");
    assert_eq!(record.account(), "assets:checking");
    assert_eq!(record.category(), "income:rsu");
}

#[test]
fn csv_row_with_all_fields_quoted_is_parsed_correctly() {
    let mapping = CsvMapping {
        timestamp_idx: 0,
        amount_idx: 1,
        memo_idx: 2,
        account_idx: 3,
        category_idx: 4,
        source_id: "test_source".to_string(),
    };
    let line = "\"2026-02-01T09:30:00\",\"12345\",\"RSU sale\",\"assets:checking\",\"income:rsu\"";

    let record = parse_simple_csv_row(line, &mapping).expect("parsed");

    assert_eq!(record.timestamp(), "2026-02-01T09:30:00");
    assert_eq!(record.amount_cents(), 12_345);
    assert_eq!(record.memo(), "RSU sale");
}

#[test]
fn csv_row_with_escaped_quotes_in_memo_is_parsed_correctly() {
    let mapping = CsvMapping::default();
    let line = "2026-02-01T09:30:00,12345,\"RSU \"\"sale\"\" vested\",assets:checking,income:rsu";

    let record = parse_simple_csv_row(line, &mapping).expect("parsed");

    assert_eq!(record.memo(), "RSU \"sale\" vested");
}

#[test]
fn csv_row_with_unterminated_quote_is_rejected() {
    let mapping = CsvMapping::default();
    let line = "2026-02-01T09:30:00,12345,\"RSU sale,assets:checking,income:rsu";

    let err = parse_simple_csv_row(line, &mapping).expect_err("must fail");

    assert_eq!(
        err,
        ImportError::InvalidCsvRow {
            message: "unterminated quoted field".to_string()
        }
    );
}

#[test]
fn csv_row_invalid_amount_reports_column_and_value() {
    let mapping = CsvMapping::default();
    let line = "2026-02-01T09:30:00,12.34,RSU sale,assets:checking,income:rsu";

    let err = parse_simple_csv_row(line, &mapping).expect_err("must fail");

    assert_eq!(
        err,
        ImportError::InvalidAmountAtColumn {
            column: 1,
            value: "12.34".to_string()
        }
    );
}

#[test]
fn test_parse_csv_columns_ignores_whitespace_after_quote() {
    let mapping = CsvMapping {
        source_id: "test".to_owned(),
        timestamp_idx: 0,
        amount_idx: 1,
        memo_idx: 2,
        account_idx: 3,
        category_idx: 4,
    };

    // The amount column is "100.00" followed by a space
    let row = r#"2026-02-01,"10000" ,"memo","assets:checking","income:imported""#;
    let result = parse_simple_csv_row(row, &mapping).unwrap();
    assert_eq!(result.amount_cents(), 10000);
}
