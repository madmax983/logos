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
    let amount_index = tokens
        .iter()
        .rposition(|token| parse_amount_cents_token(token).is_some())?;
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
    if parts.next().is_some() || !valid_month_day(month, day) {
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

    if !valid_month_day(month, day) {
        return None;
    }

    Some((year, month, day))
}

fn valid_month_day(month: u32, day: u32) -> bool {
    (1..=12).contains(&month) && (1..=31).contains(&day)
}

fn parse_amount_cents_token(token: &str) -> Option<i64> {
    let mut cleaned = token.trim_matches(|c: char| matches!(c, ',' | ';'));
    if cleaned.is_empty() {
        return None;
    }

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

    let sanitized: String = cleaned.chars().filter(|ch| *ch != ',').collect();
    if sanitized.is_empty() {
        return None;
    }

    let (whole_text, frac_text) = if let Some((whole, frac)) = sanitized.split_once('.') {
        (whole, frac)
    } else {
        (sanitized.as_str(), "")
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

    let cents = whole.checked_mul(100)?.checked_add(frac)?;
    Some(if negative { -cents } else { cents })
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
    let png_path = stem.with_extension("png");

    let pdftoppm_output = Command::new("pdftoppm")
        .arg("-f")
        .arg("1")
        .arg("-singlefile")
        .arg("-r")
        .arg("300")
        .arg("-png")
        .arg(path)
        .arg(&stem)
        .output()
        .map_err(|err| {
            if err.kind() == ErrorKind::NotFound {
                "OCR requires 'pdftoppm' and 'tesseract' on PATH".to_owned()
            } else {
                format!("pdftoppm launch failed: {err}")
            }
        })?;
    if !pdftoppm_output.status.success() {
        let stderr = String::from_utf8_lossy(&pdftoppm_output.stderr);
        return Err(format!("pdftoppm failed: {}", stderr.trim().to_owned()));
    }

    let tesseract_output = Command::new("tesseract")
        .arg(&png_path)
        .arg("stdout")
        .output()
        .map_err(|err| {
            if err.kind() == ErrorKind::NotFound {
                "OCR requires 'pdftoppm' and 'tesseract' on PATH".to_owned()
            } else {
                format!("tesseract launch failed: {err}")
            }
        })?;

    cleanup_file_if_exists(&png_path);

    if !tesseract_output.status.success() {
        let stderr = String::from_utf8_lossy(&tesseract_output.stderr);
        return Err(format!("tesseract failed: {}", stderr.trim().to_owned()));
    }

    let text = String::from_utf8_lossy(&tesseract_output.stdout).into_owned();
    if text.trim().is_empty() {
        return Err("OCR produced no textual content".to_owned());
    }
    Ok(text)
}

fn temp_ocr_stem(prefix: &str) -> PathBuf {
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0);
    std::env::temp_dir().join(format!("{prefix}-{}-{now_ms}", std::process::id()))
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
    use super::{parse_amount_cents_token, parse_date_token, parse_statement_text};

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
}
