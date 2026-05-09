use core::fmt;

/// Represents an error that occurs during a statement fetch operation.
///
/// This error type encapsulates failures such as network timeouts,
/// missing credentials, or invalid fetch configurations.
///
/// ## Examples
///
/// ```
/// use logos_fetch::FetchError;
///
/// let error = FetchError::new("network timeout during statement download");
/// assert_eq!(error.to_string(), "network timeout during statement download");
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
