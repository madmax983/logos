use core::fmt;

/// The error type returned by statement fetching operations.
///
/// This struct captures errors that occur during the fetch pipeline, such as
/// invalid configuration, failed secret resolution, or adapter execution failures.
///
/// ## Examples
///
/// ```
/// use logos_fetch::FetchError;
///
/// let err = FetchError::new("failed to parse configuration");
/// assert_eq!(err.to_string(), "failed to parse configuration");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchError {
    message: String,
}

impl FetchError {
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
