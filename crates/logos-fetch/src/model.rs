use serde::Deserialize;

use crate::FetchError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    Csv,
    Pdf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatementSource {
    source_id: String,
    institution_id: String,
    ledger_account: String,
    format_preference: Vec<OutputFormat>,
    username_secret_ref: String,
    password_secret_ref: String,
    totp_secret_ref: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FetchRunStatus {
    Downloaded,
    Imported,
    NoNewStatement,
    NeedsAttention,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchedStatementArtifact {
    source_id: String,
    ledger_account: String,
    output_format: OutputFormat,
    artifact_path: String,
    month_key: String,
    opening_balance_cents: i64,
    closing_balance_cents: i64,
}

impl StatementSource {
    /// Creates a statement source with minimal config validation.
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

pub(crate) fn is_valid_month_key(value: &str) -> bool {
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
