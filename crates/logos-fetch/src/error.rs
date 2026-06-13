use core::fmt;

/// Represents an error that occurs when fetching statements from an external source.
///
/// Fetch operations can fail due to network timeouts, invalid credentials, or changes
/// in the institution's web interface. This error type captures the failure reason
/// so that the operator can triage it (e.g., in the `fetch-run` logs).
///
/// ## Examples
///
/// ```
/// use logos_fetch::FetchError;
///
/// let err = FetchError::new("timeout connecting to chase.com");
/// assert_eq!(err.to_string(), "timeout connecting to chase.com");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchError {
    message: String,
}

impl FetchError {
    /// Creates a new `FetchError` with the given message.
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for FetchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for FetchError {}
