use logos_import::{CsvMapping, ImportError, parse_simple_csv_row};

#[test]
fn test_csv_missing_columns_usize_max_when_found_greater() {
    let mapping = CsvMapping {
        timestamp_idx: usize::MAX,
        amount_idx: usize::MAX,
        memo_idx: usize::MAX,
        account_idx: usize::MAX,
        category_idx: usize::MAX,
        source_id: "test_source".to_string(),
    };
    let line = "2026-02-01T09:30:00,12345,RSU sale,assets:checking,income:rsu";

    let err = parse_simple_csv_row(line, &mapping).unwrap_err();

    assert_eq!(
        err,
        ImportError::MissingColumns {
            expected: usize::MAX,
            found: 5,
        }
    );
}

#[test]
fn test_csv_missing_columns_usize_max_minus_one() {
    let mapping = CsvMapping {
        timestamp_idx: usize::MAX - 2,
        amount_idx: usize::MAX - 2,
        memo_idx: usize::MAX - 2,
        account_idx: usize::MAX - 2,
        category_idx: usize::MAX - 2,
        source_id: "test_source".to_string(),
    };
    let line = "2026-02-01T09:30:00,12345,RSU sale,assets:checking,income:rsu";

    let err = parse_simple_csv_row(line, &mapping).unwrap_err();

    assert_eq!(
        err,
        ImportError::MissingColumns {
            expected: usize::MAX - 1,
            found: 5,
        }
    );
}
