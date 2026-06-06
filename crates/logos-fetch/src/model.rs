//! Domain models for statement fetching and artifacts.
//!
//! This module defines the core data structures used by `logos-fetch` to describe
//! what statements should be fetched ([`StatementSource`]), the formats they are available in
//! ([`OutputFormat`]), and the resulting artifacts ([`FetchedStatementArtifact`]).

use serde::Deserialize;

use crate::FetchError;

/// The expected file format of a downloaded statement artifact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    /// Comma-separated values format.
    Csv,
    /// Portable Document Format.
    Pdf,
}

/// Configuration for a specific statement source that can be fetched.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatementSource {
    /// A unique identifier for this source configuration.
    source_id: String,
    /// The identifier of the institution this source connects to (e.g. `chase`, `amex`).
    institution_id: String,
    /// The target ledger account this statement belongs to.
    ledger_account: String,
    /// An ordered list of preferred output formats (e.g., trying CSV first, then PDF).
    format_preference: Vec<OutputFormat>,
    /// A 1Password secret reference URI pointing to the username.
    username_secret_ref: String,
    /// A 1Password secret reference URI pointing to the password.
    password_secret_ref: String,
    /// An optional 1Password secret reference URI pointing to the TOTP seed or code.
    totp_secret_ref: Option<String>,
}

/// The status resulting from a fetch operation attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FetchRunStatus {
    /// A new statement was successfully downloaded to the artifact path.
    Downloaded,
    /// The statement was already downloaded and is ready to be imported or already imported.
    Imported,
    /// The remote institution does not have a new statement available for the requested period.
    NoNewStatement,
    /// The fetch runner requires human intervention (e.g., an SMS challenge or unknown error).
    NeedsAttention,
    /// The fetch runner failed due to an error.
    Failed,
}

/// Metadata about a successfully downloaded statement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchedStatementArtifact {
    /// The ID of the source that produced this artifact.
    source_id: String,
    /// The target ledger account.
    ledger_account: String,
    /// The format of the downloaded artifact file.
    output_format: OutputFormat,
    /// The local path to the downloaded artifact file.
    artifact_path: String,
    /// The YYYY-MM month key this artifact applies to.
    month_key: String,
    /// The opening balance on the statement, in cents.
    opening_balance_cents: i64,
    /// The closing balance on the statement, in cents.
    closing_balance_cents: i64,
}

impl StatementSource {
    /// Creates a statement source with minimal config validation.
    ///
    /// # Examples
    ///
    /// ```
    /// use logos_fetch::{StatementSource, OutputFormat};
    ///
    /// let source = StatementSource::new(
    ///     "chase_checking",
    ///     "chase",
    ///     "Assets:Checking",
    ///     vec![OutputFormat::Csv]
    /// ).unwrap();
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error when any identifier is empty or no formats are configured.
    pub fn new(
        source_id: &str,
        institution_id: &str,
        ledger_account: &str,
        format_preference: Vec<OutputFormat>,
    ) -> Result<Self, FetchError> {
        if source_id.trim().is_empty() {
            return Err(FetchError::new("statement source id must not be empty"));
        }
        if institution_id.trim().is_empty() {
            return Err(FetchError::new("institution id must not be empty"));
        }
        if ledger_account.trim().is_empty() {
            return Err(FetchError::new("ledger account must not be empty"));
        }
        if format_preference.is_empty() {
            return Err(FetchError::new(
                "statement source format preference must not be empty",
            ));
        }

        Ok(Self {
            source_id: source_id.trim().to_owned(),
            institution_id: institution_id.trim().to_owned(),
            ledger_account: ledger_account.trim().to_owned(),
            format_preference,
            username_secret_ref: String::new(),
            password_secret_ref: String::new(),
            totp_secret_ref: None,
        })
    }

    #[must_use]
    pub fn source_id(&self) -> &str {
        &self.source_id
    }

    #[must_use]
    pub fn institution_id(&self) -> &str {
        &self.institution_id
    }

    #[must_use]
    pub fn ledger_account(&self) -> &str {
        &self.ledger_account
    }

    #[must_use]
    pub fn format_preference(&self) -> &[OutputFormat] {
        &self.format_preference
    }

    #[must_use]
    pub fn preferred_format(&self) -> Option<OutputFormat> {
        self.format_preference.first().copied()
    }

    /// Attaches secret references for runtime credential lookup.
    ///
    /// # Examples
    ///
    /// ```
    /// use logos_fetch::{StatementSource, OutputFormat};
    ///
    /// let source = StatementSource::new(
    ///     "chase", "chase", "Assets:Checking", vec![OutputFormat::Csv]
    /// ).unwrap()
    /// .with_secret_refs(
    ///     "op://vault/item/username",
    ///     "op://vault/item/password",
    ///     Some("op://vault/item/totp")
    /// ).unwrap();
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error when required secret references are empty or not 1Password refs.
    pub fn with_secret_refs(
        mut self,
        username_secret_ref: &str,
        password_secret_ref: &str,
        totp_secret_ref: Option<&str>,
    ) -> Result<Self, FetchError> {
        self.username_secret_ref =
            validate_secret_ref(username_secret_ref, "username secret ref", false)?;
        self.password_secret_ref =
            validate_secret_ref(password_secret_ref, "password secret ref", false)?;
        self.totp_secret_ref = totp_secret_ref
            .map(|value| validate_secret_ref(value, "totp secret ref", true))
            .transpose()?;
        Ok(self)
    }

    #[must_use]
    pub fn username_secret_ref(&self) -> &str {
        &self.username_secret_ref
    }

    #[must_use]
    pub fn password_secret_ref(&self) -> &str {
        &self.password_secret_ref
    }

    #[must_use]
    pub fn totp_secret_ref(&self) -> Option<&str> {
        self.totp_secret_ref.as_deref()
    }
}

impl FetchRunStatus {
    /// Returns `true` if the fetch run was successful.
    ///
    /// Success includes states where a new statement was downloaded,
    /// a statement was already imported, or no new statement is available yet.
    ///
    /// # Examples
    ///
    /// ```
    /// use logos_fetch::FetchRunStatus;
    ///
    /// assert!(FetchRunStatus::Downloaded.is_success());
    /// assert!(!FetchRunStatus::Failed.is_success());
    /// ```
    #[must_use]
    pub const fn is_success(self) -> bool {
        matches!(
            self,
            Self::Downloaded | Self::Imported | Self::NoNewStatement
        )
    }
}

impl FetchedStatementArtifact {
    /// Creates extracted metadata for a fetched statement artifact.
    ///
    /// # Examples
    ///
    /// ```
    /// use logos_fetch::{FetchedStatementArtifact, OutputFormat};
    ///
    /// let artifact = FetchedStatementArtifact::new(
    ///     "chase",
    ///     "Assets:Checking",
    ///     OutputFormat::Csv,
    ///     "/path/to/statement.csv",
    ///     "2023-10",
    ///     1000_00,
    ///     1500_00
    /// ).unwrap();
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error when identifiers, path, or month key are invalid.
    pub fn new(
        source_id: &str,
        ledger_account: &str,
        output_format: OutputFormat,
        artifact_path: &str,
        month_key: &str,
        opening_balance_cents: i64,
        closing_balance_cents: i64,
    ) -> Result<Self, FetchError> {
        if source_id.trim().is_empty() {
            return Err(FetchError::new(
                "fetched artifact source id must not be empty",
            ));
        }
        if ledger_account.trim().is_empty() {
            return Err(FetchError::new(
                "fetched artifact ledger account must not be empty",
            ));
        }
        if artifact_path.trim().is_empty() {
            return Err(FetchError::new("fetched artifact path must not be empty"));
        }
        if !is_valid_month_key(month_key) {
            return Err(FetchError::new(
                "fetched artifact month key must be YYYY-MM",
            ));
        }

        Ok(Self {
            source_id: source_id.trim().to_owned(),
            ledger_account: ledger_account.trim().to_owned(),
            output_format,
            artifact_path: artifact_path.trim().to_owned(),
            month_key: month_key.to_owned(),
            opening_balance_cents,
            closing_balance_cents,
        })
    }

    #[must_use]
    pub fn source_id(&self) -> &str {
        &self.source_id
    }

    #[must_use]
    pub fn ledger_account(&self) -> &str {
        &self.ledger_account
    }

    #[must_use]
    pub const fn output_format(&self) -> OutputFormat {
        self.output_format
    }

    #[must_use]
    pub fn artifact_path(&self) -> &str {
        &self.artifact_path
    }

    #[must_use]
    pub fn month_key(&self) -> &str {
        &self.month_key
    }

    #[must_use]
    pub const fn opening_balance_cents(&self) -> i64 {
        self.opening_balance_cents
    }

    #[must_use]
    pub const fn closing_balance_cents(&self) -> i64 {
        self.closing_balance_cents
    }
}

/// Checks if a given string is a valid month key in the format `YYYY-MM`.
///
/// This is used extensively to validate month-based partitioning and reporting keys
/// before passing them down to the ledger.
///
/// ## Examples
///
/// ```rust,ignore
/// use logos_fetch::model::is_valid_month_key;
///
/// assert!(is_valid_month_key("2024-05"));
/// assert!(is_valid_month_key("1999-12"));
///
/// // Invalid formats
/// assert!(!is_valid_month_key("2024-13")); // Month out of range
/// assert!(!is_valid_month_key("24-05")); // Year too short
/// assert!(!is_valid_month_key("2024/05")); // Wrong separator
/// ```
pub fn is_valid_month_key(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 7 || bytes[4] != b'-' {
        return false;
    }

    if !bytes[0..4].iter().all(u8::is_ascii_digit) || !bytes[5..7].iter().all(u8::is_ascii_digit) {
        return false;
    }

    matches!(value[5..7].parse::<u8>(), Ok(1..=12))
}

fn validate_secret_ref(
    value: &str,
    field_name: &str,
    allow_empty: bool,
) -> Result<String, FetchError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        if allow_empty {
            return Ok(String::new());
        }
        return Err(FetchError::new(format!("{field_name} must not be empty")));
    }
    if !trimmed.starts_with("op://") {
        return Err(FetchError::new(format!(
            "{field_name} must be a 1Password reference"
        )));
    }
    Ok(trimmed.to_owned())
}
