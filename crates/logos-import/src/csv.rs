use core::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportError {
    MissingColumns { expected: usize, found: usize },
    InvalidAmount,
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

/// Parses a single CSV row into an import record with deterministic field mapping.
///
/// # Errors
///
/// Returns an error when required columns are missing or amount parsing fails.
pub fn parse_simple_csv_row(row: &str, mapping: &CsvMapping) -> Result<ImportRecord, ImportError> {
    let columns: Vec<&str> = row.split(',').collect();
    let needed = 1 + [
        mapping.timestamp_idx,
        mapping.amount_idx,
        mapping.memo_idx,
        mapping.account_idx,
        mapping.category_idx,
    ]
    .into_iter()
    .max()
    .unwrap_or(0);

    if columns.len() < needed {
        return Err(ImportError::MissingColumns {
            expected: needed,
            found: columns.len(),
        });
    }

    let amount_cents = columns[mapping.amount_idx]
        .trim()
        .parse::<i64>()
        .map_err(|_| ImportError::InvalidAmount)?;

    Ok(ImportRecord::new(
        &mapping.source_id,
        columns[mapping.timestamp_idx].trim(),
        amount_cents,
        columns[mapping.memo_idx].trim(),
        columns[mapping.account_idx].trim(),
        columns[mapping.category_idx].trim(),
    ))
}
