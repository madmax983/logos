use core::future::Future;
use core::pin::Pin;
use std::path::PathBuf;

use crate::{
    FetchError, FetchRunStatus, FetchedStatementArtifact, OutputFormat, SecretBundle,
    StatementSource, model::is_valid_month_key,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchRequest {
    source: StatementSource,
    month_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchResult {
    status: FetchRunStatus,
    artifact: Option<FetchedStatementArtifact>,
    error_summary: Option<String>,
}

pub trait StatementAdapter {
    fn fetch<'a>(
        &'a self,
        request: &'a FetchRequest,
        secrets: &'a SecretBundle,
    ) -> Pin<Box<dyn Future<Output = Result<FetchResult, FetchError>> + Send + 'a>>;
}

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
    pub fn source(&self) -> &StatementSource {
        &self.source
    }

    #[must_use]
    pub fn month_key(&self) -> &str {
        &self.month_key
    }
}

impl FetchResult {
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
    pub fn artifact(&self) -> Option<&FetchedStatementArtifact> {
        self.artifact.as_ref()
    }

    /// Attaches an operator-facing summary for non-success fetch results.
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
