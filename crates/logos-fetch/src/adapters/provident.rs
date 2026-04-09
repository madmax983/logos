//! Implementation of the Provident statement adapter.
//!
//! This adapter wraps the execution and output parsing of the external `provident` tool.

use core::future::Future;
use core::pin::Pin;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::{
    FetchError, FetchRequest, FetchResult, FetchRunStatus, FetchedStatementArtifact, OutputFormat,
    SecretBundle, StatementAdapter, model::is_valid_month_key,
};

/// An adapter that integrates with the external `provident` tool via JSON output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProvidentAdapter {
    runner_output_path: PathBuf,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ProvidentRunnerStatus {
    Downloaded,
    Imported,
    NoNewStatement,
    NeedsAttention,
    Failed,
}

#[derive(Debug, Deserialize)]
struct ProvidentRunnerOutput {
    status: ProvidentRunnerStatus,
    artifact_path: Option<String>,
    statement_month: Option<String>,
    output_format: Option<OutputFormat>,
    opening_balance_cents: Option<i64>,
    closing_balance_cents: Option<i64>,
    error_summary: Option<String>,
}

impl ProvidentAdapter {
    /// Creates a Provident adapter pointing to a test fixture.
    #[must_use]
    pub fn fixture_runner_output() -> Self {
        Self {
            runner_output_path: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("tests")
                .join("fixtures")
                .join("provident_statement_metadata.json"),
        }
    }

    /// Creates a Provident adapter from one runner output JSON file.
    ///
    /// # Examples
    ///
    /// ```
    /// use logos_fetch::ProvidentAdapter;
    ///
    /// let adapter = ProvidentAdapter::from_runner_output_path("output.json").unwrap();
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error when the provided path is empty.
    pub fn from_runner_output_path(path: impl AsRef<Path>) -> Result<Self, FetchError> {
        let path = path.as_ref();
        if path.as_os_str().is_empty() {
            return Err(FetchError::new(
                "provident runner output path must not be empty",
            ));
        }

        Ok(Self {
            runner_output_path: path.to_path_buf(),
        })
    }

    fn load_runner_output(&self) -> Result<ProvidentRunnerOutput, FetchError> {
        let input = fs::read_to_string(&self.runner_output_path).map_err(|err| {
            FetchError::new(format!(
                "failed to read provident runner output '{}': {err}",
                self.runner_output_path.display()
            ))
        })?;

        serde_json::from_str(&input).map_err(|err| {
            FetchError::new(format!(
                "invalid provident runner output '{}': {err}",
                self.runner_output_path.display()
            ))
        })
    }

    fn map_runner_output(
        &self,
        request: &FetchRequest,
        output: &ProvidentRunnerOutput,
    ) -> Result<FetchResult, FetchError> {
        match output.status {
            ProvidentRunnerStatus::Downloaded => {
                self.map_downloaded_result(request, FetchRunStatus::Downloaded, output)
            }
            ProvidentRunnerStatus::Imported => {
                self.map_downloaded_result(request, FetchRunStatus::Imported, output)
            }
            ProvidentRunnerStatus::NoNewStatement => {
                Ok(FetchResult::new(FetchRunStatus::NoNewStatement, None))
            }
            ProvidentRunnerStatus::NeedsAttention => Self::map_failure_result(
                FetchRunStatus::NeedsAttention,
                output.error_summary.as_deref(),
            ),
            ProvidentRunnerStatus::Failed => {
                Self::map_failure_result(FetchRunStatus::Failed, output.error_summary.as_deref())
            }
        }
    }

    fn map_downloaded_result(
        &self,
        request: &FetchRequest,
        status: FetchRunStatus,
        output: &ProvidentRunnerOutput,
    ) -> Result<FetchResult, FetchError> {
        let artifact_path = self.resolve_artifact_path(required_field(
            output.artifact_path.as_deref(),
            "artifact_path",
        )?);
        let month_key = normalize_statement_month(required_field(
            output.statement_month.as_deref(),
            "statement_month",
        )?)?;
        if month_key != request.month_key() {
            return Err(FetchError::new(format!(
                "provident runner output month '{}' does not match request month '{}'",
                month_key,
                request.month_key()
            )));
        }

        let artifact = FetchedStatementArtifact::new(
            request.source().source_id(),
            request.source().ledger_account(),
            output.output_format.unwrap_or(OutputFormat::Pdf),
            &artifact_path,
            &month_key,
            required_i64(output.opening_balance_cents, "opening_balance_cents")?,
            required_i64(output.closing_balance_cents, "closing_balance_cents")?,
        )?;

        Ok(FetchResult::new(status, Some(artifact)))
    }

    fn map_failure_result(
        status: FetchRunStatus,
        error_summary: Option<&str>,
    ) -> Result<FetchResult, FetchError> {
        FetchResult::new(status, None).with_error_summary(
            error_summary.unwrap_or("provident runner did not include an error summary"),
        )
    }

    fn resolve_artifact_path(&self, artifact_path: &str) -> String {
        let artifact_path = PathBuf::from(artifact_path);
        if artifact_path.is_absolute() {
            return artifact_path.to_string_lossy().to_string();
        }

        self.runner_output_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(artifact_path)
            .to_string_lossy()
            .to_string()
    }
}

impl StatementAdapter for ProvidentAdapter {
    fn fetch<'a>(
        &'a self,
        request: &'a FetchRequest,
        _secrets: &'a SecretBundle,
    ) -> Pin<Box<dyn Future<Output = Result<FetchResult, FetchError>> + Send + 'a>> {
        Box::pin(async move {
            let output = self.load_runner_output()?;
            self.map_runner_output(request, &output)
        })
    }
}

fn required_field<'a>(value: Option<&'a str>, field_name: &str) -> Result<&'a str, FetchError> {
    let Some(value) = value.map(str::trim) else {
        return Err(FetchError::new(format!(
            "provident runner output missing '{field_name}'"
        )));
    };
    if value.is_empty() {
        return Err(FetchError::new(format!(
            "provident runner output field '{field_name}' must not be empty"
        )));
    }
    Ok(value)
}

fn required_i64(value: Option<i64>, field_name: &str) -> Result<i64, FetchError> {
    value.ok_or_else(|| FetchError::new(format!("provident runner output missing '{field_name}'")))
}

fn normalize_statement_month(value: &str) -> Result<String, FetchError> {
    let trimmed = value.trim();
    if is_valid_month_key(trimmed) {
        return Ok(trimmed.to_owned());
    }

    let bytes = trimmed.as_bytes();
    let is_valid_date = bytes.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes[0..4].iter().all(u8::is_ascii_digit)
        && bytes[5..7].iter().all(u8::is_ascii_digit)
        && bytes[8..10].iter().all(u8::is_ascii_digit);
    if is_valid_date {
        let month_key = trimmed[..7].to_owned();
        if is_valid_month_key(&month_key) {
            return Ok(month_key);
        }
    }

    Err(FetchError::new(format!(
        "provident runner output month '{trimmed}' must be YYYY-MM or YYYY-MM-DD"
    )))
}
