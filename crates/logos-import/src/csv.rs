//! CSV Import Handling
use core::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportError {
    MissingColumns { expected: usize, found: usize },
    InvalidAmount,
    InvalidAmountAtColumn { column: usize, value: String },
    InvalidCsvRow { message: String },
    FileReadFailed { path: String, message: String },
    PdfTextExtractionFailed { path: String, message: String },
    NoStatementRows { path: String },
}

impl fmt::Display for ImportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingColumns { expected, found } => {
                write!(f, "missing columns: expected {expected}, found {found}")
            }
            Self::InvalidAmount => write!(f, "invalid amount in CSV row"),
            Self::InvalidAmountAtColumn { column, value } => {
                write!(f, "invalid amount in CSV row at column {column}: '{value}'")
            }
            Self::InvalidCsvRow { message } => write!(f, "invalid CSV row: {message}"),
            Self::FileReadFailed { path, message } => {
                write!(f, "failed reading import file '{path}': {message}")
            }
            Self::PdfTextExtractionFailed { path, message } => {
                write!(f, "failed extracting text from PDF '{path}': {message}")
            }
            Self::NoStatementRows { path } => {
                write!(f, "no statement rows parsed from '{path}'")
            }
        }
    }
}

impl std::error::Error for ImportError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CsvMapping {
    pub source_id: String,
    pub timestamp_idx: usize,
    pub amount_idx: usize,
    pub memo_idx: usize,
    pub account_idx: usize,
    pub category_idx: usize,
}

impl Default for CsvMapping {
    fn default() -> Self {
        Self {
            source_id: "manual.csv".to_owned(),
            timestamp_idx: 0,
            amount_idx: 1,
            memo_idx: 2,
            account_idx: 3,
            category_idx: 4,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportRecord {
    source_id: String,
    timestamp: String,
    amount_cents: i64,
    memo: String,
    account: String,
    category: String,
}

impl ImportRecord {
    #[must_use]
    pub fn new(
        source_id: &str,
        timestamp: &str,
        amount_cents: i64,
        memo: &str,
        account: &str,
        category: &str,
    ) -> Self {
        Self {
            source_id: source_id.to_owned(),
            timestamp: timestamp.to_owned(),
            amount_cents,
            memo: memo.to_owned(),
            account: account.to_owned(),
            category: category.to_owned(),
        }
    }

    #[must_use]
    pub fn source_id(&self) -> &str {
        &self.source_id
    }

    #[must_use]
    pub fn timestamp(&self) -> &str {
        &self.timestamp
    }

    #[must_use]
    pub const fn amount_cents(&self) -> i64 {
        self.amount_cents
    }

    #[must_use]
    pub fn memo(&self) -> &str {
        &self.memo
    }

    #[must_use]
    pub fn account(&self) -> &str {
        &self.account
    }

    #[must_use]
    pub fn category(&self) -> &str {
        &self.category
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CsvFieldState {
    Unquoted,
    Quoted,
    AfterQuote,
}

/// ⚡ Bolt Optimization: Uses `Vec::with_capacity` and `String::with_capacity`
/// to avoid repeated heap allocations and dynamic resizing while assembling
/// CSV fields character-by-character. 8 columns and 32 chars are reasonable
/// empirical defaults for standard financial statement data.
fn parse_csv_columns(row: &str) -> Result<Vec<String>, ImportError> {
    let mut columns = Vec::with_capacity(8);
    let mut field = String::with_capacity(32);
    let mut state = CsvFieldState::Unquoted;
    let mut chars = row.chars().peekable();

    while let Some(ch) = chars.next() {
        match state {
            CsvFieldState::Unquoted => match ch {
                ',' => {
                    columns.push(field);
                    field = String::with_capacity(32);
                }
                '"' => {
                    if field.trim().is_empty() {
                        field.clear();
                        state = CsvFieldState::Quoted;
                    } else {
                        return Err(ImportError::InvalidCsvRow {
                            message: "unexpected quote in unquoted field".to_string(),
                        });
                    }
                }
                _ => field.push(ch),
            },
            CsvFieldState::Quoted => {
                if ch == '"' {
                    if matches!(chars.peek(), Some('"')) {
                        let _ = chars.next();
                        field.push('"');
                    } else {
                        state = CsvFieldState::AfterQuote;
                    }
                } else {
                    field.push(ch);
                }
            }
            CsvFieldState::AfterQuote => match ch {
                ',' => {
                    columns.push(field);
                    field = String::with_capacity(32);
                    state = CsvFieldState::Unquoted;
                }
                _ if ch.is_whitespace() => {}
                _ => {
                    return Err(ImportError::InvalidCsvRow {
                        message: "unexpected characters after closing quote".to_string(),
                    });
                }
            },
        }
    }

    if state == CsvFieldState::Quoted {
        return Err(ImportError::InvalidCsvRow {
            message: "unterminated quoted field".to_string(),
        });
    }

    columns.push(field);
    Ok(columns)
}

/// Parses a single CSV row into an import record with deterministic field mapping.
///
/// # Errors
///
/// Returns an error when CSV quoting is malformed, required columns are missing,
/// or amount parsing fails.
pub fn parse_simple_csv_row(row: &str, mapping: &CsvMapping) -> Result<ImportRecord, ImportError> {
    let columns = parse_csv_columns(row)?;
    let max_idx = [
        mapping.timestamp_idx,
        mapping.amount_idx,
        mapping.memo_idx,
        mapping.account_idx,
        mapping.category_idx,
    ]
    .into_iter()
    .max()
    .unwrap_or(0);

    let needed = max_idx.saturating_add(1);

    // If max_idx is usize::MAX, needed becomes usize::MAX because of saturating_add.
    // We need at least max_idx + 1 columns to safely access max_idx.
    // If max_idx is usize::MAX, we can never have enough columns (since columns.len() <= usize::MAX),
    // so it's guaranteed to be MissingColumns or out of bounds.
    if max_idx == usize::MAX || columns.len() < needed {
        // If max_idx is MAX, the 'expected' value conceptually exceeds usize,
        // we can just cap it at usize::MAX for the error reporting.
        return Err(ImportError::MissingColumns {
            expected: if max_idx == usize::MAX {
                usize::MAX
            } else {
                needed
            },
            found: columns.len(),
        });
    }

    let amount_value = columns[mapping.amount_idx].trim();
    let amount_cents =
        amount_value
            .parse::<i64>()
            .map_err(|_| ImportError::InvalidAmountAtColumn {
                column: mapping.amount_idx,
                value: amount_value.to_owned(),
            })?;

    Ok(ImportRecord::new(
        &mapping.source_id,
        columns[mapping.timestamp_idx].trim(),
        amount_cents,
        columns[mapping.memo_idx].trim(),
        columns[mapping.account_idx].trim(),
        columns[mapping.category_idx].trim(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_csv_columns_table() {
        struct TestCase {
            name: &'static str,
            input: &'static str,
            expected: Result<Vec<&'static str>, ImportError>,
        }

        let cases = vec![
            TestCase {
                name: "normal fields",
                input: "a,b,c",
                expected: Ok(vec!["a", "b", "c"]),
            },
            TestCase {
                name: "quoted fields",
                input: "\"a\",\"b\",\"c\"",
                expected: Ok(vec!["a", "b", "c"]),
            },
            TestCase {
                name: "quoted field with comma",
                input: "a,\"b,c\",d",
                expected: Ok(vec!["a", "b,c", "d"]),
            },
            TestCase {
                name: "escaped quotes",
                input: "a,\"b\"\"c\",d",
                expected: Ok(vec!["a", "b\"c", "d"]),
            },
            TestCase {
                name: "empty field at end",
                input: "a,b,",
                expected: Ok(vec!["a", "b", ""]),
            },
            TestCase {
                name: "empty field at start",
                input: ",a,b",
                expected: Ok(vec!["", "a", "b"]),
            },
            TestCase {
                name: "all empty fields",
                input: ",,",
                expected: Ok(vec!["", "", ""]),
            },
            TestCase {
                name: "whitespace before quote is ignored",
                input: "a, \"b\",c",
                expected: Ok(vec!["a", "b", "c"]),
            },
            TestCase {
                name: "whitespace after quote is ignored",
                input: "a,\"b\" ,c",
                expected: Ok(vec!["a", "b", "c"]),
            },
            TestCase {
                name: "unterminated quote",
                input: "a,\"b,c",
                expected: Err(ImportError::InvalidCsvRow {
                    message: "unterminated quoted field".to_string(),
                }),
            },
            TestCase {
                name: "unexpected quote in unquoted field",
                input: "a,b\"c,d",
                expected: Err(ImportError::InvalidCsvRow {
                    message: "unexpected quote in unquoted field".to_string(),
                }),
            },
            TestCase {
                name: "unexpected characters after closing quote",
                input: "a,\"b\"c,d",
                expected: Err(ImportError::InvalidCsvRow {
                    message: "unexpected characters after closing quote".to_string(),
                }),
            },
        ];

        for case in cases {
            let actual = parse_csv_columns(case.input);
            match case.expected {
                Ok(expected) => {
                    let actual = actual.unwrap();
                    let expected_strings: Vec<String> =
                        expected.into_iter().map(String::from).collect();
                    assert_eq!(actual, expected_strings, "Failed test case: {}", case.name);
                }
                Err(expected_err) => {
                    let err = actual.unwrap_err();
                    assert_eq!(err, expected_err, "Failed test case: {}", case.name);
                }
            }
        }
    }
}
