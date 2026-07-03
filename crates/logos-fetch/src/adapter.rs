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

    /// Provides access to the underlying [`StatementSource`] configuring this run.
    ///
    /// This acts as the identity binding, telling the adapter which institution
    /// to target and which ledger account the resulting statement belongs to.
    ///
    /// ## Examples
    ///
    /// ```
    /// use logos_fetch::{FetchRequest, StatementSource, OutputFormat};
    /// let source = StatementSource::new("chase", "chase", "Assets:Checking", vec![OutputFormat::Csv]).unwrap();
    /// let request = FetchRequest::new(&source, "2023-10").unwrap();
    /// assert_eq!(request.source().institution_id(), "chase");
    /// ```
    #[must_use]
    pub const fn source(&self) -> &StatementSource {
        &self.source
    }

    /// Exposes the specific `YYYY-MM` accounting period the fetch is targeting.
    ///
    /// Adapters use this to query only the transactions relevant to the active
    /// reconciliation cycle, ignoring older historical data.
    ///
    /// ## Examples
    ///
    /// ```
    /// use logos_fetch::{FetchRequest, StatementSource, OutputFormat};
    /// let source = StatementSource::new("chase", "chase", "Assets:Checking", vec![OutputFormat::Csv]).unwrap();
    /// let request = FetchRequest::new(&source, "2023-10").unwrap();
    /// assert_eq!(request.month_key(), "2023-10");
    /// ```
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

    /// Extracts the finalized [`FetchRunStatus`] summarizing the adapter execution.
    ///
    /// This is the primary control flow value for the fetch pipeline, indicating
    /// whether a new artifact is available or if human intervention is required.
    ///
    /// ## Examples
    ///
    /// ```
    /// use logos_fetch::{FetchResult, FetchRunStatus};
    /// let result = FetchResult::new(FetchRunStatus::NoNewStatement, None);
    /// assert_eq!(result.status(), FetchRunStatus::NoNewStatement);
    /// ```
    #[must_use]
    pub const fn status(&self) -> FetchRunStatus {
        self.status
    }

    /// Retrieves the generated `FetchedStatementArtifact`, if one was created.
    ///
    /// This will only be `Some` if the `status` is [`FetchRunStatus::Downloaded`]
    /// or [`FetchRunStatus::Imported`].
    ///
    /// ## Examples
    ///
    /// ```
    /// use logos_fetch::{FetchResult, FetchRunStatus};
    /// let result = FetchResult::new(FetchRunStatus::NeedsAttention, None);
    /// assert!(result.artifact().is_none());
    /// ```
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

    /// Reads the optional operator-facing error summary string.
    ///
    /// This provides context when a fetch fails or needs attention, such as
    /// detailing SMS multi-factor auth challenges.
    ///
    /// ## Examples
    ///
    /// ```
    /// use logos_fetch::{FetchResult, FetchRunStatus};
    /// let result = FetchResult::new(FetchRunStatus::NeedsAttention, None).with_error_summary("SMS Code Required").unwrap();
    /// assert_eq!(result.error_summary(), Some("SMS Code Required"));
    /// ```
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
