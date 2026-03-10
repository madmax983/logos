use serde::Deserialize;

use crate::{FetchError, OutputFormat, StatementSource};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatementSourceConfig {
    sources: Vec<StatementSource>,
}

#[derive(Debug, Deserialize)]
struct RawStatementSourceConfig {
    sources: Vec<RawStatementSource>,
}

#[derive(Debug, Deserialize)]
struct RawStatementSource {
    source_id: String,
    institution_id: String,
    ledger_account: String,
    format_preference: Vec<OutputFormat>,
    username_secret_ref: String,
    password_secret_ref: String,
    #[serde(default)]
    totp_secret_ref: Option<String>,
}

impl StatementSourceConfig {
    /// Parses statement source config from TOML text.
    ///
    /// # Errors
    ///
    /// Returns an error when TOML is invalid or a source fails validation.
    pub fn from_toml(input: &str) -> Result<Self, FetchError> {
        let raw: RawStatementSourceConfig = toml::from_str(input)
            .map_err(|err| FetchError::new(format!("invalid config: {err}")))?;
        if raw.sources.is_empty() {
            return Err(FetchError::new(
                "statement source config must contain at least one source",
            ));
        }

        let mut sources = Vec::with_capacity(raw.sources.len());
        for source in raw.sources {
            let statement_source = StatementSource::new(
                &source.source_id,
                &source.institution_id,
                &source.ledger_account,
                source.format_preference,
            )?
            .with_secret_refs(
                &source.username_secret_ref,
                &source.password_secret_ref,
                source.totp_secret_ref.as_deref(),
            )?;
            sources.push(statement_source);
        }

        Ok(Self { sources })
    }

    #[must_use]
    pub fn sources(&self) -> &[StatementSource] {
        &self.sources
    }
}
