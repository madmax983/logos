use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use logos_import::parse_pdf_statement_file;

fn temp_file_path(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!("logos-import-{prefix}-{nanos}.pdf"))
}

fn cleanup_file(path: &Path) {
    if path.exists() {
        let _ = std::fs::remove_file(path);
    }
}

#[test]
fn parses_plain_text_statement_file_into_records() {
    let path = temp_file_path("plain-text");
    std::fs::write(
        &path,
        "2026-02-01 COFFEE SHOP -12.34\n2026-02-02 PAYROLL 1000.00\n",
    )
    .expect("write");

    let records = parse_pdf_statement_file(&path, "assets:checking", false).expect("parsed");
    assert_eq!(records.len(), 2);
    assert_eq!(records[0].amount_cents(), -1_234);
    assert_eq!(records[1].amount_cents(), 100_000);

    cleanup_file(&path);
}

#[test]
fn rejects_when_no_statement_rows_are_present() {
    let path = temp_file_path("no-rows");
    std::fs::write(&path, "HEADER ONLY\nPAGE 1\n").expect("write");

    let err = parse_pdf_statement_file(&path, "assets:checking", false).expect_err("must fail");
    assert_eq!(
        err.to_string(),
        format!("no statement rows parsed from '{}'", path.display())
    );

    cleanup_file(&path);
}

#[test]
fn parses_plain_text_with_ocr_flag_enabled() {
    let path = temp_file_path("plain-text-ocr-flag");
    std::fs::write(&path, "2026-02-01 BOOK STORE -20.00\n").expect("write");

    let records = parse_pdf_statement_file(&path, "assets:checking", true).expect("parsed");
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].amount_cents(), -2_000);

    cleanup_file(&path);
}

#[test]
fn pdf_parse_date_and_amount_token_distances_swapped() {
    let path = temp_file_path("token-distances-swapped");
    // date index and amount index are swapped
    std::fs::write(&path, "-12.34 2026-02-01\n").expect("write");

    let err = parse_pdf_statement_file(&path, "assets:checking", false).expect_err("must fail");
    assert_eq!(
        err.to_string(),
        format!("no statement rows parsed from '{}'", path.display())
    );

    cleanup_file(&path);
}

#[test]
fn pdf_statement_iso_dates_with_invalid_parts() {
    let path = temp_file_path("invalid-iso-dates");
    // This will hit the valid_month_day function
    std::fs::write(&path, "2026-13-01 COFFEE -12.34\n2026-00-30 TEA -5.00\n").expect("write");

    let err = parse_pdf_statement_file(&path, "assets:checking", false).expect_err("must fail");
    assert_eq!(
        err.to_string(),
        format!("no statement rows parsed from '{}'", path.display())
    );

    cleanup_file(&path);
}

#[test]
fn pdf_statement_iso_dates_with_too_many_parts() {
    let path = temp_file_path("too-many-parts-iso-dates");
    // This will hit the valid_month_day function
    std::fs::write(&path, "2026-12-01-01 COFFEE -12.34\n").expect("write");

    let err = parse_pdf_statement_file(&path, "assets:checking", false).expect_err("must fail");
    assert_eq!(
        err.to_string(),
        format!("no statement rows parsed from '{}'", path.display())
    );

    cleanup_file(&path);
}

#[test]
fn pdf_statement_slash_dates_with_invalid_parts() {
    let path = temp_file_path("invalid-slash-dates");
    // This will hit the valid_month_day function
    std::fs::write(&path, "13/01/2026 COFFEE -12.34\n00/30/2026 TEA -5.00\n").expect("write");

    let err = parse_pdf_statement_file(&path, "assets:checking", false).expect_err("must fail");
    assert_eq!(
        err.to_string(),
        format!("no statement rows parsed from '{}'", path.display())
    );

    cleanup_file(&path);
}

#[test]
fn pdf_statement_amount_token_dollars_only() {
    let path = temp_file_path("amount-dollars-only");
    std::fs::write(&path, "2026-02-01 COFFEE -12\n").expect("write");

    let records = parse_pdf_statement_file(&path, "assets:checking", false).expect("must succeed");
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].amount_cents(), -1200);

    cleanup_file(&path);
}

#[test]
fn pdf_statement_amount_token_commas() {
    let path = temp_file_path("amount-dollars-commas");
    std::fs::write(&path, "2026-02-01 COFFEE 1,234.56\n").expect("write");

    let records = parse_pdf_statement_file(&path, "assets:checking", false).expect("must succeed");
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].amount_cents(), 123_456);

    cleanup_file(&path);
}

#[test]
fn pdf_statement_amount_token_leading_dollar() {
    let path = temp_file_path("amount-dollars-leading");
    std::fs::write(&path, "2026-02-01 COFFEE -$12.34\n").expect("write");

    let records = parse_pdf_statement_file(&path, "assets:checking", false).expect("must succeed");
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].amount_cents(), -1234);

    cleanup_file(&path);
}

#[test]
fn pdf_statement_amount_token_decimals_bad() {
    let path = temp_file_path("amount-decimals-bad");
    std::fs::write(&path, "2026-02-01 COFFEE 12.345\n").expect("write");

    let err = parse_pdf_statement_file(&path, "assets:checking", false).expect_err("must fail");
    assert_eq!(
        err.to_string(),
        format!("no statement rows parsed from '{}'", path.display())
    );

    cleanup_file(&path);
}

#[test]
fn pdf_parse_date_and_amount_token_distances() {
    let path = temp_file_path("token-distances");
    // amount index is date index + 1 (memo is empty)
    std::fs::write(&path, "2026-02-01 -12.34\n").expect("write");

    // date index and amount index are swapped
    std::fs::write(&path, "2026-02-01 -12.34\n").expect("write");

    let err = parse_pdf_statement_file(&path, "assets:checking", false).expect_err("must fail");
    assert_eq!(
        err.to_string(),
        format!("no statement rows parsed from '{}'", path.display())
    );

    cleanup_file(&path);
}

#[test]
fn pdf_parse_zero_memo() {
    let path = temp_file_path("zero-memo");
    // memo is only whitespace
    std::fs::write(&path, "2026-02-01     -12.34\n").expect("write");

    let err = parse_pdf_statement_file(&path, "assets:checking", false).expect_err("must fail");
    assert_eq!(
        err.to_string(),
        format!("no statement rows parsed from '{}'", path.display())
    );

    cleanup_file(&path);
}

#[test]
fn pdf_parse_statement_line_with_empty_lines() {
    let path = temp_file_path("empty-lines");
    std::fs::write(
        &path,
        "2026-02-01 TXN -12.34\n    \n\n2026-02-02 TXN -12.34\n",
    )
    .expect("write");
    let records = parse_pdf_statement_file(&path, "assets:checking", false).expect("must succeed");
    assert_eq!(records.len(), 2);
    cleanup_file(&path);
}

#[test]
fn pdf_parse_statement_line_with_short_lines() {
    let path = temp_file_path("short-lines");
    // lines with fewer than 3 tokens
    std::fs::write(&path, "2026-02-01 -12.34\n2026-02-02 TXN\nTXN -12.34\n").expect("write");
    let err = parse_pdf_statement_file(&path, "assets:checking", false).expect_err("must fail");
    assert_eq!(
        err.to_string(),
        format!("no statement rows parsed from '{}'", path.display())
    );
    cleanup_file(&path);
}

#[test]
fn pdf_parse_date_and_amount_token_distances_swapped_exact2() {
    let path = temp_file_path("token-distances-swapped-exact2");
    // date index and amount index are at exact offset to test < and <=
    // if replaced + with *, 0 + 1 = 1, 0 * 1 = 0
    // so let's put date at 2
    // 2 + 1 = 3, 2 * 1 = 2
    std::fs::write(&path, "X Y 2026-02-01 -12.34 TXN\n").expect("write");

    let err = parse_pdf_statement_file(&path, "assets:checking", false).expect_err("must fail");
    assert_eq!(
        err.to_string(),
        format!("no statement rows parsed from '{}'", path.display())
    );

    cleanup_file(&path);
}

#[test]
fn pdf_parse_statement_file_empty_ocr_result() {
    let path = temp_file_path("empty-ocr.pdf");
    // enable_ocr = true
    // we don't have pdftotext or tesseract so it'll probably fail with extraction error
    let _err = parse_pdf_statement_file(&path, "assets:checking", true).expect_err("must fail");
    cleanup_file(&path);
}

#[test]
fn pdf_parse_iso_date_month_day_validity() {
    // Tests valid_month_day logic more precisely
    let path = temp_file_path("iso-month-day");

    // Day 0
    std::fs::write(&path, "2026-02-00 TXN 12.34\n").expect("write");
    assert!(parse_pdf_statement_file(&path, "assets:checking", false).is_err());

    // Month 0
    std::fs::write(&path, "2026-00-01 TXN 12.34\n").expect("write");
    assert!(parse_pdf_statement_file(&path, "assets:checking", false).is_err());

    // Month 13
    std::fs::write(&path, "2026-13-01 TXN 12.34\n").expect("write");
    assert!(parse_pdf_statement_file(&path, "assets:checking", false).is_err());

    // Day 32
    std::fs::write(&path, "2026-02-32 TXN 12.34\n").expect("write");
    assert!(parse_pdf_statement_file(&path, "assets:checking", false).is_err());

    cleanup_file(&path);
}
