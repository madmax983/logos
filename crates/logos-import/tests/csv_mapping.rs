use logos_import::{CsvMapping, parse_simple_csv_row};

#[test]
fn csv_row_is_mapped_into_import_record() {
    let mapping = CsvMapping::default();
    let line = "2026-02-01T09:30:00,12345,RSU sale,assets:checking,income:rsu";

    let record = parse_simple_csv_row(line, &mapping).expect("parsed");

    assert_eq!(record.timestamp(), "2026-02-01T09:30:00");
    assert_eq!(record.amount_cents(), 12_345);
    assert_eq!(record.memo(), "RSU sale");
    assert_eq!(record.account(), "assets:checking");
    assert_eq!(record.category(), "income:rsu");
}
