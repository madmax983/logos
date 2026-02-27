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
