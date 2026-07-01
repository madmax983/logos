use core::fmt;

/// A strongly typed error representing failures during a statement fetch operation.
///
/// This encompasses configuration parsing failures, secret resolution errors, and
/// actual adapter download failures.
///
/// # Recovery
/// If you encounter a `FetchError` during an automated run, the recommended recovery
/// is to alert the operator, as these are typically environmental issues (e.g.,
/// 1Password CLI not authenticated, incorrect configuration, or website layout changes).
///
/// ## Examples
///
/// ```
/// use logos_fetch::FetchError;
///
/// let error = FetchError::new("failed to execute 1Password CLI");
/// assert_eq!(error.to_string(), "failed to execute 1Password CLI");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchError {
    message: String,
}

impl FetchError {
    /// Creates a new `FetchError` with the provided message.
    ///
    /// ## Examples
    ///
    /// ```
    /// use logos_fetch::FetchError;
    ///
    /// let error = FetchError::new("configuration file not found");
    /// ```
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
