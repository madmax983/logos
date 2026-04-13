use core::fmt;

/// Represents the ways an import operation can fail.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportError {
    /// The CSV row did not have the required number of columns.
    MissingColumns {
        /// The expected minimum number of columns.
        expected: usize,
        /// The actual number of columns found.
        found: usize,
    },
    /// The amount field in the CSV row was invalid or missing.
    InvalidAmount,
    /// The amount field at a specific column could not be parsed.
    InvalidAmountAtColumn {
        /// The 0-based index of the column containing the invalid amount.
        column: usize,
        /// The string value that failed to parse as an integer amount in cents.
        value: String,
    },
    /// The CSV row had a malformed syntax (e.g. unterminated quotes).
    InvalidCsvRow {
        /// A descriptive message of what was wrong with the row syntax.
        message: String,
    },
    /// The file could not be read from the filesystem.
    FileReadFailed {
        /// The path of the file that was attempted to be read.
        path: String,
        /// The underlying OS error message.
        message: String,
    },
    /// Text extraction from a PDF document failed.
    PdfTextExtractionFailed {
        /// The path of the PDF file.
        path: String,
        /// The specific error from the PDF extraction engine.
        message: String,
    },
    /// The document was parsed successfully but did not contain any matching statement rows.
    NoStatementRows {
        /// The path to the document that contained no statement records.
        path: String,
    },
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

/// Represents the column indices for a generic CSV mapping.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CsvMapping {
    /// A unique identifier for the source system generating this CSV.
    pub source_id: String,
    /// The 0-based column index where the date/time is located.
    pub timestamp_idx: usize,
    /// The 0-based column index where the transaction amount (in cents) is located.
    pub amount_idx: usize,
    /// The 0-based column index where the transaction description or memo is located.
    pub memo_idx: usize,
    /// The 0-based column index where the account name is located.
    pub account_idx: usize,
    /// The 0-based column index where the category is located.
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

/// An intermediate representation of a single transaction parsed from an external file.
///
/// This type ensures all required fields are present before attempting to construct
/// a full `Transaction` via the core builder, which enforces double-entry rules.
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
    /// Creates a new `ImportRecord` from raw, parsed fields.
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

    /// The source system identifier.
    #[must_use]
    pub fn source_id(&self) -> &str {
        &self.source_id
    }

    /// The raw timestamp string from the import source.
    #[must_use]
    pub fn timestamp(&self) -> &str {
        &self.timestamp
    }

    /// The parsed transaction amount in cents.
    #[must_use]
    pub const fn amount_cents(&self) -> i64 {
        self.amount_cents
    }

    /// The description or memo attached to the transaction.
    #[must_use]
    pub fn memo(&self) -> &str {
        &self.memo
    }

    /// The primary account name affected by this transaction.
    #[must_use]
    pub fn account(&self) -> &str {
        &self.account
    }

    /// The category group or categorization tag.
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

fn parse_csv_columns(row: &str) -> Result<Vec<String>, ImportError> {
    let mut columns = Vec::new();
    let mut field = String::new();
    let mut state = CsvFieldState::Unquoted;
    let mut chars = row.chars().peekable();

    while let Some(ch) = chars.next() {
        match state {
            CsvFieldState::Unquoted => match ch {
                ',' => {
                    columns.push(field);
                    field = String::new();
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
                    field = String::new();
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
