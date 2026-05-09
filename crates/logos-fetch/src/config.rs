//! Configuration models for statement sources.
//!
//! This module provides parsing and deserialization for the overall fetch configuration,
//! mapping TOML files into an executable list of [`StatementSource`] items.

use serde::Deserialize;

use crate::{FetchError, OutputFormat, StatementSource};

/// The aggregated configuration of all statement sources.
///
/// This struct holds the parsed TOML definitions specifying which
/// external accounts should be fetched, and provides an iterable
/// list of [`StatementSource`] items for the runtime to execute.
///
/// ## Examples
///
/// ```
/// use logos_fetch::StatementSourceConfig;
///
/// let toml = r#"
/// [[sources]]
/// source_id = "chase_checking"
/// institution_id = "chase"
/// ledger_account = "Assets:Checking"
/// format_preference = ["csv", "pdf"]
/// username_secret_ref = "op://vault/item/username"
/// password_secret_ref = "op://vault/item/password"
/// "#;
///
/// let config = StatementSourceConfig::from_toml(toml).unwrap();
/// assert_eq!(config.sources()[0].source_id(), "chase_checking");
/// ```
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
    /// Parses a complete statement source configuration from a TOML document.
    ///
    /// # Examples
    ///
    /// ```
    /// use logos_fetch::StatementSourceConfig;
    ///
    /// let toml = r#"
    /// [[sources]]
    /// source_id = "chase_checking"
    /// institution_id = "chase"
    /// ledger_account = "Assets:Checking"
    /// format_preference = ["csv", "pdf"]
    /// username_secret_ref = "op://vault/item/username"
    /// password_secret_ref = "op://vault/item/password"
    /// "#;
    ///
    /// let config = StatementSourceConfig::from_toml(toml).unwrap();
    /// assert_eq!(config.sources().len(), 1);
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error when the TOML syntax is invalid, if required fields are missing,
    /// or if a source fails internal validation.
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
