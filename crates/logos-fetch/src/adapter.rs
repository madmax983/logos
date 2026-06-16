//! Traits and models for adapters that fetch financial statements.
//!
//! This module defines the `StatementAdapter` interface that must be
//! implemented to perform statement downloads from an external institution.
//! It also provides the core I/O models: [`FetchRequest`] and [`FetchResult`].

use core::future::Future;
use core::pin::Pin;
use std::path::PathBuf;

use crate::{
    FetchError, FetchRunStatus, FetchedStatementArtifact, OutputFormat, SecretBundle,
    StatementSource, model::is_valid_month_key,
};

/// A request to fetch a statement from a source for a specific month.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchRequest {
    source: StatementSource,
    month_key: String,
}

/// The result of attempting to fetch a statement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchResult {
    status: FetchRunStatus,
    artifact: Option<FetchedStatementArtifact>,
    error_summary: Option<String>,
}

/// A trait defining the contract for adapters capable of downloading statements.
pub trait StatementAdapter {
    /// Attempt to fetch a statement for the specified request using the provided credentials.
    fn fetch<'a>(
        &'a self,
        request: &'a FetchRequest,
        secrets: &'a SecretBundle,
    ) -> Pin<Box<dyn Future<Output = Result<FetchResult, FetchError>> + Send + 'a>>;
}

/// A fake implementation of [`StatementAdapter`] for testing purposes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FakeStatementAdapter {
    mode: FakeStatementAdapterMode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum FakeStatementAdapterMode {
    DownloadFixture { artifact_path: String },
    NeedsAttention { error_summary: String },
}

impl FetchRequest {
    /// Creates a fetch request for one source and month.
    ///
    /// # Examples
    ///
    /// ```
    /// use logos_fetch::{FetchRequest, StatementSource, OutputFormat};
    ///
    /// let source = StatementSource::new(
    ///     "chase", "chase", "Assets:Checking", vec![OutputFormat::Csv]
    /// ).unwrap();
    /// let request = FetchRequest::new(&source, "2023-10").unwrap();
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error when the month key is invalid.
    pub fn new(source: &StatementSource, month_key: &str) -> Result<Self, FetchError> {
        if !is_valid_month_key(month_key) {
            return Err(FetchError::new("fetch request month key must be YYYY-MM"));
        }

        Ok(Self {
            source: source.clone(),
            month_key: month_key.to_owned(),
        })
    }

    #[must_use]
    pub const fn source(&self) -> &StatementSource {
        &self.source
    }

    #[must_use]
    pub fn month_key(&self) -> &str {
        &self.month_key
    }
}

impl FetchResult {
    /// Creates a new fetch result.
    #[must_use]
    pub const fn new(status: FetchRunStatus, artifact: Option<FetchedStatementArtifact>) -> Self {
        Self {
            status,
            artifact,
            error_summary: None,
        }
    }

    #[must_use]
    pub const fn status(&self) -> FetchRunStatus {
        self.status
    }

    #[must_use]
    pub const fn artifact(&self) -> Option<&FetchedStatementArtifact> {
        self.artifact.as_ref()
    }

    /// Attaches an operator-facing summary for non-success fetch results.
    ///
    /// # Examples
    ///
    /// ```
    /// use logos_fetch::{FetchResult, FetchRunStatus};
    ///
    /// let result = FetchResult::new(FetchRunStatus::NeedsAttention, None)
    ///     .with_error_summary("Requires SMS OTP challenge").unwrap();
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error when the provided summary is empty.
    pub fn with_error_summary(mut self, error_summary: &str) -> Result<Self, FetchError> {
        let trimmed = error_summary.trim();
        if trimmed.is_empty() {
            return Err(FetchError::new(
                "fetch result error summary must not be empty",
            ));
        }

        self.error_summary = Some(trimmed.to_owned());
        Ok(self)
    }

    #[must_use]
    pub fn error_summary(&self) -> Option<&str> {
        self.error_summary.as_deref()
    }
}

impl FakeStatementAdapter {
    /// Creates a fake adapter that successfully "downloads" a test fixture.
    ///
    /// It only supports the `2026-02` statement month.
    #[must_use]
    pub fn download_fixture_statement() -> Self {
        Self {
            mode: FakeStatementAdapterMode::DownloadFixture {
                artifact_path: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("tests")
                    .join("fixtures")
                    .join("fake-statement-2026-02.pdf")
                    .to_string_lossy()
                    .to_string(),
            },
        }
    }

    /// Creates a fake adapter that always requires manual operator attention.
    #[must_use]
    pub fn needs_attention(error_summary: &str) -> Self {
        let trimmed = error_summary.trim();
        let error_summary = if trimmed.is_empty() {
            "manual attention required"
        } else {
            trimmed
        };
        Self {
            mode: FakeStatementAdapterMode::NeedsAttention {
                error_summary: error_summary.to_owned(),
            },
        }
    }
}

impl StatementAdapter for FakeStatementAdapter {
    fn fetch<'a>(
        &'a self,
        request: &'a FetchRequest,
        _secrets: &'a SecretBundle,
    ) -> Pin<Box<dyn Future<Output = Result<FetchResult, FetchError>> + Send + 'a>> {
        Box::pin(async move {
            match &self.mode {
                FakeStatementAdapterMode::DownloadFixture { artifact_path } => {
                    if request.month_key() != "2026-02" {
                        return Err(FetchError::new(format!(
                            "fake fixture only supports statement month 2026-02, got {}",
                            request.month_key()
                        )));
                    }
                    let artifact = FetchedStatementArtifact::new(
                        request.source().source_id(),
                        request.source().ledger_account(),
                        OutputFormat::Pdf,
                        artifact_path,
                        request.month_key(),
                        100_000,
                        198_766,
                    )?;

                    Ok(FetchResult::new(FetchRunStatus::Downloaded, Some(artifact)))
                }
                FakeStatementAdapterMode::NeedsAttention { error_summary } => {
                    FetchResult::new(FetchRunStatus::NeedsAttention, None)
                        .with_error_summary(error_summary)
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_build_fetch_request_with_correct_properties() -> Result<(), crate::error::FetchError> {
        let mut source = crate::model::StatementSource::new(
            "src_1", "inst_1", "acct_1", vec![crate::model::OutputFormat::Csv]
        )?;
        source = source.with_secret_refs("op://user", "op://pass", None)?;

        let req = FetchRequest::new(&source, "2023-01")?;
        assert_eq!(req.month_key(), "2023-01");
        assert_eq!(req.source().source_id(), "src_1");
        Ok(())
    }

    #[test]
    fn should_return_error_when_month_key_is_invalid() {
        let source = crate::model::StatementSource::new(
            "src_1", "inst_1", "acct_1", vec![crate::model::OutputFormat::Csv]
        ).unwrap();
        let err = FetchRequest::new(&source, "invalid").unwrap_err();
        assert_eq!(err.to_string(), "fetch request month key must be YYYY-MM");
    }

    #[test]
    fn should_build_fetch_result_with_artifact() -> Result<(), crate::error::FetchError> {
        let result_success = FetchResult::new(
            crate::model::FetchRunStatus::Downloaded,
            Some(crate::model::FetchedStatementArtifact::new(
                "src_1", "acct_1", crate::model::OutputFormat::Csv, "path/to/csv", "2023-01", 100, 200
            )?)
        );

        assert_eq!(result_success.status(), crate::model::FetchRunStatus::Downloaded);
        assert!(result_success.artifact().is_some());
        assert_eq!(result_success.error_summary(), None);
        Ok(())
    }

    #[test]
    fn should_build_fetch_result_without_artifact() {
        let result_imported = FetchResult::new(crate::model::FetchRunStatus::Imported, None);
        assert_eq!(result_imported.status(), crate::model::FetchRunStatus::Imported);
        assert!(result_imported.artifact().is_none());

        let result_no_stmt = FetchResult::new(crate::model::FetchRunStatus::NoNewStatement, None);
        assert_eq!(result_no_stmt.status(), crate::model::FetchRunStatus::NoNewStatement);
        assert!(result_no_stmt.artifact().is_none());
    }

    #[test]
    fn should_build_fetch_result_with_needs_attention() -> Result<(), crate::error::FetchError> {
        let result_attn = FetchResult::new(crate::model::FetchRunStatus::NeedsAttention, None)
            .with_error_summary("needs 2FA")?;
        assert_eq!(result_attn.status(), crate::model::FetchRunStatus::NeedsAttention);
        assert_eq!(result_attn.error_summary(), Some("needs 2FA"));
        Ok(())
    }

    #[test]
    fn should_return_error_when_needs_attention_summary_is_empty() {
        let err = FetchResult::new(crate::model::FetchRunStatus::NeedsAttention, None)
            .with_error_summary("  ").unwrap_err();
        assert_eq!(err.to_string(), "fetch result error summary must not be empty");
    }
}
