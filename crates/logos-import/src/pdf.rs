//! PDF Statement Parsing
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::csv::{ImportError, ImportRecord};

/// Parses statement-like rows from a PDF file into import records.
///
/// For text PDFs, this prefers `pdftotext` when available and falls back
/// to extracting PDF literal strings directly from file bytes.
///
/// # Errors
///
/// Returns an error when file reading, text extraction, or row parsing fails.
pub fn parse_pdf_statement_file(
    path: impl AsRef<Path>,
    account: &str,
    enable_ocr: bool,
) -> Result<Vec<ImportRecord>, ImportError> {
    let path_ref = path.as_ref();
    let source_id = path_ref
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("statement.pdf");
    let display_path = path_ref.display().to_string();
    let bytes = fs::read(path_ref).map_err(|err| ImportError::FileReadFailed {
        path: display_path.clone(),
        message: err.to_string(),
    })?;

    let is_pdf = bytes.starts_with(b"%PDF-");
    let text = if is_pdf {
        match extract_pdf_text(path_ref, &bytes) {
            Ok(text) => text,
            Err(err) => {
                if enable_ocr {
                    String::new()
                } else {
                    return Err(err);
                }
            }
        }
    } else {
        String::from_utf8_lossy(&bytes).into_owned()
    };
    let mut records = parse_statement_text(&text, source_id, account);
    if !records.is_empty() {
        return Ok(records);
    }

    if is_pdf && enable_ocr {
        let ocr_text = extract_pdf_text_with_ocr(path_ref).map_err(|message| {
            ImportError::PdfTextExtractionFailed {
                path: display_path.clone(),
                message,
            }
        })?;
        records = parse_statement_text(&ocr_text, source_id, account);
        if !records.is_empty() {
            return Ok(records);
        }
    }

    Err(ImportError::NoStatementRows { path: display_path })
}

fn parse_statement_text(text: &str, source_id: &str, account: &str) -> Vec<ImportRecord> {
    text.lines()
        .filter_map(|line| parse_statement_line(line, source_id, account))
        .collect()
}

fn parse_statement_line(line: &str, source_id: &str, account: &str) -> Option<ImportRecord> {
    let compact = line.trim();
    if compact.is_empty() {
        return None;
    }

    let tokens: Vec<&str> = compact.split_whitespace().collect();
    if tokens.len() < 3 {
        return None;
    }

    let date_index = tokens
        .iter()
        .position(|token| parse_date_token(token).is_some())?;
    let amount_index = select_amount_index(&tokens, date_index)?;
    if amount_index <= date_index + 1 {
        return None;
    }

    let timestamp = parse_date_token(tokens[date_index])?;
    let amount_cents = parse_amount_cents_token(tokens[amount_index])?;
    let memo = tokens[(date_index + 1)..amount_index].join(" ");
    if memo.trim().is_empty() {
        return None;
    }

    let category = if amount_cents < 0 {
        "expenses:imported"
    } else {
        "income:imported"
    };

    Some(ImportRecord::new(
        source_id,
        &timestamp,
        amount_cents,
        &memo,
        account,
        category,
    ))
}

/// Selects the index of the token that represents the transaction amount.
/// Avoids heap allocations by tracking only the last two matching indices
/// iteratively rather than collecting them into an intermediate vector.
fn select_amount_index(tokens: &[&str], date_index: usize) -> Option<usize> {
    let mut last = None;
    let mut previous = None;

    for (index, token) in tokens.iter().enumerate().skip(date_index + 1) {
        if parse_amount_cents_token(token).is_some() {
            previous = last;
            last = Some(index);
        }
    }

    let mut amount_index = last?;

    // Common statements encode "... <txn amount> <running balance>" as adjacent columns.
    if let (Some(l), Some(p)) = (last, previous) {
        if l == p + 1 && is_unsigned_amount_token(tokens[l]) {
            amount_index = p;
        }
    }

    Some(amount_index)
}

fn is_unsigned_amount_token(token: &str) -> bool {
    let cleaned = token.trim_matches(|c: char| matches!(c, ',' | ';'));
    parse_amount_cents_token(cleaned).is_some()
        && !cleaned.starts_with('-')
        && !cleaned.starts_with('+')
        && !cleaned.starts_with('(')
}

fn parse_date_token(token: &str) -> Option<String> {
    let cleaned = token.trim_matches(|c: char| matches!(c, ',' | ';'));
    if cleaned.is_empty() {
        return None;
    }

    if let Some((year, month, day)) = parse_iso_date(cleaned) {
        return Some(format!("{year:04}-{month:02}-{day:02}T00:00:00"));
    }
    if let Some((year, month, day)) = parse_slash_date(cleaned) {
        return Some(format!("{year:04}-{month:02}-{day:02}T00:00:00"));
    }

    None
}

fn parse_iso_date(token: &str) -> Option<(u32, u32, u32)> {
    let mut parts = token.split('-');
    let year = parts.next()?.parse::<u32>().ok()?;
    let month = parts.next()?.parse::<u32>().ok()?;
    let day = parts.next()?.parse::<u32>().ok()?;
    if parts.next().is_some() || !valid_calendar_date(year, month, day) {
        return None;
    }
    Some((year, month, day))
}

fn parse_slash_date(token: &str) -> Option<(u32, u32, u32)> {
    let mut parts = token.split('/');
    let month = parts.next()?.parse::<u32>().ok()?;
    let day = parts.next()?.parse::<u32>().ok()?;
    let year_raw = parts.next()?.parse::<u32>().ok()?;
    if parts.next().is_some() {
        return None;
    }

    let year = if year_raw < 100 {
        2_000_u32.checked_add(year_raw)?
    } else {
        year_raw
    };

    if !valid_calendar_date(year, month, day) {
        return None;
    }

    Some((year, month, day))
}

fn valid_calendar_date(year: u32, month: u32, day: u32) -> bool {
    if year == 0 {
        return false;
    }

    let max_day = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => return false,
    };
    (1..=max_day).contains(&day)
}

const fn is_leap_year(year: u32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn parse_amount_cents_token(token: &str) -> Option<i64> {
    let cleaned = token.trim_matches(|c: char| matches!(c, ',' | ';'));
    if cleaned.is_empty() {
        return None;
    }

    let (negative, numeric_part) = strip_amount_sign_and_currency(cleaned);

    let sanitized: String = numeric_part.chars().filter(|ch| *ch != ',').collect();
    if sanitized.is_empty() {
        return None;
    }

    let cents = parse_cents_from_sanitized(&sanitized)?;
    Some(if negative { -cents } else { cents })
}

fn strip_amount_sign_and_currency(mut cleaned: &str) -> (bool, &str) {
    let mut negative = false;
    if let Some(inner) = cleaned
        .strip_prefix('(')
        .and_then(|value| value.strip_suffix(')'))
    {
        negative = true;
        cleaned = inner;
    }
    if let Some(value) = cleaned.strip_prefix('-') {
        negative = true;
        cleaned = value;
    } else if let Some(value) = cleaned.strip_prefix('+') {
        cleaned = value;
    }
    if let Some(value) = cleaned.strip_prefix('$') {
        cleaned = value;
    }
    (negative, cleaned)
}

fn parse_cents_from_sanitized(sanitized: &str) -> Option<i64> {
    let (whole_text, frac_text) = if let Some((whole, frac)) = sanitized.split_once('.') {
        (whole, frac)
    } else {
        (sanitized, "")
    };
    if whole_text.is_empty() || frac_text.len() > 2 {
        return None;
    }

    let whole = whole_text.parse::<i64>().ok()?;
    let frac = if frac_text.is_empty() {
        0_i64
    } else if frac_text.len() == 1 {
        i64::from(frac_text.chars().next()?.to_digit(10)?) * 10
    } else {
        frac_text.parse::<i64>().ok()?
    };

    whole.checked_mul(100)?.checked_add(frac)
}

fn extract_pdf_text(path: &Path, bytes: &[u8]) -> Result<String, ImportError> {
    if let Some(text) = extract_with_pdftotext(path) {
        if !text.trim().is_empty() {
            return Ok(text);
        }
    }

    let text = extract_pdf_literal_strings(bytes);
    if !text.trim().is_empty() {
        return Ok(text);
    }

    Err(ImportError::PdfTextExtractionFailed {
        path: path.display().to_string(),
        message: "no textual content extracted".to_owned(),
    })
}

fn extract_pdf_text_with_ocr(path: &Path) -> Result<String, String> {
    let stem = temp_ocr_stem("logos-pdf-ocr");
    let png_paths = run_pdftoppm(path, &stem)?;

    let text_result = (|| {
        let mut text = String::new();
        for png_path in &png_paths {
            let page_text = run_tesseract(png_path)?;
            let trimmed = page_text.trim();
            if trimmed.is_empty() {
                continue;
            }
            if !text.is_empty() {
                text.push('\n');
            }
            text.push_str(trimmed);
        }
        Ok::<String, String>(text)
    })();

    for png_path in &png_paths {
        cleanup_file_if_exists(png_path);
    }

    let text = text_result?;
    if text.trim().is_empty() {
        return Err("OCR produced no textual content".to_owned());
    }
    Ok(text)
}

fn run_pdftoppm(path: &Path, stem: &Path) -> Result<Vec<PathBuf>, String> {
    let output = Command::new("pdftoppm")
        .arg("-r")
        .arg("300")
        .arg("-png")
        .arg(path)
        .arg(stem)
        .output()
        .map_err(|err| {
            if err.kind() == ErrorKind::NotFound {
                "OCR requires 'pdftoppm' and 'tesseract' on PATH".to_owned()
            } else {
                format!("pdftoppm launch failed: {err}")
            }
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("pdftoppm failed: {}", stderr.trim().to_owned()));
    }

    collect_ocr_page_images(stem)
}

fn run_tesseract(png_path: &Path) -> Result<String, String> {
    let output = Command::new("tesseract")
        .arg(png_path)
        .arg("stdout")
        .output()
        .map_err(|err| {
            if err.kind() == ErrorKind::NotFound {
                "OCR requires 'pdftoppm' and 'tesseract' on PATH".to_owned()
            } else {
                format!("tesseract launch failed: {err}")
            }
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("tesseract failed: {}", stderr.trim().to_owned()));
    }

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn temp_ocr_stem(prefix: &str) -> PathBuf {
    let now_ns = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    std::env::temp_dir().join(format!("{prefix}-{}-{now_ns}", std::process::id()))
}

fn cleanup_file_if_exists(path: &Path) {
    if path.exists() {
        let _ = fs::remove_file(path);
    }
}

fn extract_with_pdftotext(path: &Path) -> Option<String> {
    let output = Command::new("pdftotext")
        .arg("-layout")
        .arg("-nopgbrk")
        .arg(path)
        .arg("-")
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    Some(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn collect_ocr_page_images(stem: &Path) -> Result<Vec<PathBuf>, String> {
    let directory = stem
        .parent()
        .ok_or_else(|| "unable to resolve OCR temporary directory".to_owned())?;
    let stem_name = stem
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| "unable to resolve OCR temporary file stem".to_owned())?;
    let prefix = format!("{stem_name}-");

    let mut page_images = Vec::new();
    let entries = fs::read_dir(directory)
        .map_err(|err| format!("failed listing OCR temporary directory: {err}"))?;
    for entry in entries {
        let entry = entry.map_err(|err| format!("failed reading OCR temporary entry: {err}"))?;
        let path = entry.path();
        let Some(file_name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        let is_png = Path::new(file_name)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("png"));
        if !is_png || !file_name.starts_with(&prefix) {
            continue;
        }
        page_images.push(path);
    }

    page_images.sort_by(|left, right| {
        let left_name = left
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or_default();
        let right_name = right
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or_default();
        let left_page = parse_pdftoppm_page_number(left_name, stem_name).unwrap_or(u32::MAX);
        let right_page = parse_pdftoppm_page_number(right_name, stem_name).unwrap_or(u32::MAX);
        left_page
            .cmp(&right_page)
            .then_with(|| left_name.cmp(right_name))
    });

    if page_images.is_empty() {
        return Err("pdftoppm produced no PNG output".to_owned());
    }
    Ok(page_images)
}

fn parse_pdftoppm_page_number(file_name: &str, stem_name: &str) -> Option<u32> {
    file_name
        .strip_prefix(stem_name)?
        .strip_prefix('-')?
        .strip_suffix(".png")?
        .parse::<u32>()
        .ok()
}

fn extract_pdf_literal_strings(bytes: &[u8]) -> String {
    let mut out = String::new();
    let mut current = String::new();
    let mut in_string = false;
    let mut escaped = false;

    for byte in bytes {
        let ch = char::from(*byte);
        if !in_string {
            if ch == '(' {
                in_string = true;
                current.clear();
                escaped = false;
            }
            continue;
        }

        if escaped {
            let mapped = match ch {
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                'b' => '\u{0008}',
                'f' => '\u{000C}',
                '(' => '(',
                ')' => ')',
                '\\' => '\\',
                other => other,
            };
            current.push(mapped);
            escaped = false;
            continue;
        }

        match ch {
            '\\' => escaped = true,
            ')' => {
                let fragment = current.trim();
                if !fragment.is_empty() {
                    if !out.is_empty() {
                        out.push('\n');
                    }
                    out.push_str(fragment);
                }
                in_string = false;
            }
            other => current.push(other),
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{
        collect_ocr_page_images, parse_amount_cents_token, parse_date_token, parse_statement_text,
    };

    #[test]
    fn parses_date_token_variants() {
        assert_eq!(
            parse_date_token("2026-02-27").as_deref(),
            Some("2026-02-27T00:00:00")
        );
        assert_eq!(
            parse_date_token("02/27/2026").as_deref(),
            Some("2026-02-27T00:00:00")
        );
        assert_eq!(
            parse_date_token("02/27/26").as_deref(),
            Some("2026-02-27T00:00:00")
        );
    }

    #[test]
    fn parses_amount_tokens() {
        assert_eq!(parse_amount_cents_token("$1,234.56"), Some(123_456));
        assert_eq!(parse_amount_cents_token("-12.34"), Some(-1_234));
        assert_eq!(parse_amount_cents_token("(99.99)"), Some(-9_999));
    }

    #[test]
    fn parses_statement_text_into_import_records() {
        let text = "\
2026-02-01 COFFEE SHOP -12.34
2026-02-02 PAYROLL 1000.00";
        let records = parse_statement_text(text, "stmt.pdf", "assets:checking");

        assert_eq!(records.len(), 2);
        assert_eq!(records[0].memo(), "COFFEE SHOP");
        assert_eq!(records[0].amount_cents(), -1_234);
        assert_eq!(records[0].category(), "expenses:imported");
        assert_eq!(records[1].amount_cents(), 100_000);
        assert_eq!(records[1].category(), "income:imported");
    }

    #[test]
    fn parses_statement_lines_with_trailing_balance_column() {
        let text = "\
2026-02-01 COFFEE SHOP -12.34 1,234.56
2026-02-02 PAYROLL 1000.00 2,234.56";
        let records = parse_statement_text(text, "stmt.pdf", "assets:checking");

        assert_eq!(records.len(), 2);
        assert_eq!(records[0].memo(), "COFFEE SHOP");
        assert_eq!(records[0].amount_cents(), -1_234);
        assert_eq!(records[1].memo(), "PAYROLL");
        assert_eq!(records[1].amount_cents(), 100_000);
    }

    #[test]
    fn rejects_non_calendar_dates_and_accepts_leap_day() {
        assert_eq!(parse_date_token("2026-02-31"), None);
        assert_eq!(parse_date_token("02/29/2025"), None);
        assert_eq!(
            parse_date_token("2024-02-29").as_deref(),
            Some("2024-02-29T00:00:00")
        );
        assert_eq!(
            parse_date_token("02/29/2024").as_deref(),
            Some("2024-02-29T00:00:00")
        );
    }

    #[test]
    fn collects_ocr_images_in_page_order() {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let stem = std::env::temp_dir().join(format!("logos-import-ocr-pages-{nanos}"));
        let page_10 = stem.with_file_name(format!(
            "{}-10.png",
            stem.file_name()
                .and_then(|value| value.to_str())
                .expect("stem")
        ));
        let page_2 = stem.with_file_name(format!(
            "{}-2.png",
            stem.file_name()
                .and_then(|value| value.to_str())
                .expect("stem")
        ));
        let page_1 = stem.with_file_name(format!(
            "{}-1.png",
            stem.file_name()
                .and_then(|value| value.to_str())
                .expect("stem")
        ));
        std::fs::write(&page_10, []).expect("write page 10");
        std::fs::write(&page_2, []).expect("write page 2");
        std::fs::write(&page_1, []).expect("write page 1");

        let pages = collect_ocr_page_images(&stem).expect("collect pages");
        assert_eq!(pages, vec![page_1.clone(), page_2.clone(), page_10.clone()]);

        let _ = std::fs::remove_file(page_1);
        let _ = std::fs::remove_file(page_2);
        let _ = std::fs::remove_file(page_10);
    }
}
