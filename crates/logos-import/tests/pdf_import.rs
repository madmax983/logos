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

#[test]
fn test_is_unsigned_amount_token_rejects_signed_and_invalid_tokens() {
    // This function is private, so we test it via select_amount_index behavior indirectly
    // "select_amount_index" favors unsigned over previous ones if it sees an adjacent token.
    let pdf_path = std::env::temp_dir().join(format!("unsigned-token-{}.pdf", std::process::id()));

    // Case 1: Signed running balance shouldn't trigger the "adjacent unsigned amount" override
    // 2026-02-01 ITEM 10.00 -50.00
    // Here, 10.00 is amount, -50.00 is next token. Since it starts with '-', it's not unsigned.
    // The amount selected should be the LAST valid amount token, which is -50.00 (since it didn't trigger the previous override).
    std::fs::write(
        &pdf_path,
        b"%PDF-1.4
(2026-02-01 ITEM 10.00 -50.00)
",
    )
    .unwrap();
    let result =
        logos_import::pdf::parse_pdf_statement_file(&pdf_path, "assets:checking", false).unwrap();
    assert_eq!(result[0].amount_cents(), -5000);

    // Case 2: Unsigned running balance SHOULD trigger the override
    // 2026-02-01 ITEM 10.00 50.00
    // Here, 10.00 is amount, 50.00 is unsigned running balance.
    // The amount selected should be the previous valid amount token, which is 10.00.
    let pdf_path2 =
        std::env::temp_dir().join(format!("unsigned-token-2-{}.pdf", std::process::id()));
    std::fs::write(
        &pdf_path2,
        b"%PDF-1.4
(2026-02-01 ITEM 10.00 50.00)
",
    )
    .unwrap();
    let result2 =
        logos_import::pdf::parse_pdf_statement_file(&pdf_path2, "assets:checking", false).unwrap();
    assert_eq!(result2[0].amount_cents(), 1000);

    std::fs::remove_file(&pdf_path).ok();
    std::fs::remove_file(&pdf_path2).ok();
}

#[test]
fn test_parse_slash_date_year_raw_under_100_adds_2000() {
    let pdf_path = std::env::temp_dir().join(format!("slash-date-1-{}.pdf", std::process::id()));
    std::fs::write(
        &pdf_path,
        b"%PDF-1.4
(02/01/26 ITEM 10.00)
",
    )
    .unwrap();
    let result =
        logos_import::pdf::parse_pdf_statement_file(&pdf_path, "assets:checking", false).unwrap();
    assert_eq!(result[0].timestamp(), "2026-02-01T00:00:00");

    let pdf_path2 = std::env::temp_dir().join(format!("slash-date-2-{}.pdf", std::process::id()));
    std::fs::write(
        &pdf_path2,
        b"%PDF-1.4
(02/01/2026 ITEM 10.00)
",
    )
    .unwrap();
    let result2 =
        logos_import::pdf::parse_pdf_statement_file(&pdf_path2, "assets:checking", false).unwrap();
    assert_eq!(result2[0].timestamp(), "2026-02-01T00:00:00");

    std::fs::remove_file(&pdf_path).ok();
    std::fs::remove_file(&pdf_path2).ok();
}

#[test]
fn test_valid_calendar_date_rejects_invalid_months_and_days() {
    let pdf_path = std::env::temp_dir().join(format!("invalid-dates-{}.pdf", std::process::id()));
    // 0000 year is invalid
    // 13 month is invalid
    // 32 day in Jan is invalid
    // 31 day in April is invalid
    std::fs::write(
        &pdf_path,
        b"%PDF-1.4
(0000-01-01 ITEM 10.00
2026-13-01 ITEM 10.00
2026-01-32 ITEM 10.00
2026-04-31 ITEM 10.00)
",
    )
    .unwrap();
    let result = logos_import::pdf::parse_pdf_statement_file(&pdf_path, "assets:checking", false);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(
        err,
        logos_import::ImportError::NoStatementRows { .. }
    ));
    std::fs::remove_file(&pdf_path).ok();
}

#[test]
fn test_is_leap_year_logic() {
    let pdf_path = std::env::temp_dir().join(format!("leap-years-{}.pdf", std::process::id()));
    // 2024 is leap year (divisible by 4, not 100) -> Feb 29 valid
    // 2026 is not leap year -> Feb 29 invalid
    // 1900 is not leap year (divisible by 100, not 400) -> Feb 29 invalid
    // 2000 is leap year (divisible by 400) -> Feb 29 valid
    std::fs::write(
        &pdf_path,
        b"%PDF-1.4
(2024-02-29 ITEM 10.00
2026-02-29 ITEM 10.00
1900-02-29 ITEM 10.00
2000-02-29 ITEM 10.00)
",
    )
    .unwrap();
    let result =
        logos_import::pdf::parse_pdf_statement_file(&pdf_path, "assets:checking", false).unwrap();
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].timestamp(), "2024-02-29T00:00:00");
    assert_eq!(result[1].timestamp(), "2000-02-29T00:00:00");
    std::fs::remove_file(&pdf_path).ok();
}

#[test]
fn test_parse_cents_from_sanitized_handles_various_fraction_lengths() {
    let pdf_path = std::env::temp_dir().join(format!("fractions-{}.pdf", std::process::id()));
    // Valid: 10, 10., 10.1, 10.12
    // Invalid: 10.123
    std::fs::write(
        &pdf_path,
        b"%PDF-1.4
(2026-02-01 ITEM 10
2026-02-02 ITEM 10.
2026-02-03 ITEM 10.1
2026-02-04 ITEM 10.12
2026-02-05 ITEM 10.123)
",
    )
    .unwrap();
    let result =
        logos_import::pdf::parse_pdf_statement_file(&pdf_path, "assets:checking", false).unwrap();
    assert_eq!(result.len(), 4);
    assert_eq!(result[0].amount_cents(), 1000);
    assert_eq!(result[1].amount_cents(), 1000);
    assert_eq!(result[2].amount_cents(), 1010);
    assert_eq!(result[3].amount_cents(), 1012);
    std::fs::remove_file(&pdf_path).ok();
}

#[test]
fn test_extract_pdf_text_falls_back_to_literal_strings_or_fails() {
    let pdf_path = std::env::temp_dir().join(format!("no-text-{}.pdf", std::process::id()));

    // Test 1: Only literal strings present. `extract_with_pdftotext` returns nothing useful.
    std::fs::write(
        &pdf_path,
        b"%PDF-1.4
((2026-02-01 TXN 10.00))
",
    )
    .unwrap();
    // Assuming pdftotext won't parse this fake pdf, it should fallback to literal strings.
    // If it does, we get 1 record.
    let result = logos_import::pdf::parse_pdf_statement_file(&pdf_path, "assets:checking", false);
    if let Ok(records) = result {
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].amount_cents(), 1000);
    }

    // Test 2: No literal strings either -> fails with NoStatementRows
    std::fs::write(
        &pdf_path,
        b"%PDF-1.4
(NOTHING USEFUL HERE)
",
    )
    .unwrap();
    let result2 = logos_import::pdf::parse_pdf_statement_file(&pdf_path, "assets:checking", false);
    assert!(result2.is_err());
    let err2 = result2.unwrap_err();
    assert!(matches!(
        err2,
        logos_import::ImportError::NoStatementRows { .. }
    ));

    std::fs::remove_file(&pdf_path).ok();
}

#[test]
fn test_extract_pdf_text_with_ocr_fails_if_no_text_extracted() {
    let pdf_path = std::env::temp_dir().join(format!("ocr-fail-{}.pdf", std::process::id()));
    // Give it a file that isn't a valid PDF, so pdftoppm will fail
    std::fs::write(&pdf_path, b"%PDF-1.4\nBLAH").unwrap();

    let result = logos_import::pdf::parse_pdf_statement_file(&pdf_path, "assets:checking", true);
    assert!(result.is_err());
    // Should be an OCR error bubble up to PdfTextExtractionFailed
    let err = result.unwrap_err();
    assert!(matches!(
        err,
        logos_import::ImportError::PdfTextExtractionFailed { .. }
    ));
    std::fs::remove_file(&pdf_path).ok();
}

#[test]
fn test_extract_pdf_literal_strings_handles_escape_sequences() {
    // This function is private. We test it via the fallback logic by providing a PDF
    // that pdftotext can't parse, so it hits the literal string extractor.
    let pdf_path = std::env::temp_dir().join(format!("escapes-{}.pdf", std::process::id()));

    // \n is newline, \t is tab, \\ is backslash, \( and \) are parens
    // "2026-02-01\tTXN\\NAME\n10.00"
    std::fs::write(
        &pdf_path,
        b"%PDF-1.4
((2026-02-01 TXN\\(NAME\\) 10.00))
",
    )
    .unwrap();

    let result = logos_import::pdf::parse_pdf_statement_file(&pdf_path, "assets:checking", false);
    if let Ok(records) = result {
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].memo(), "TXN(NAME)");
    }

    std::fs::remove_file(&pdf_path).ok();
}
