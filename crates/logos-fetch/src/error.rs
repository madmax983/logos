use core::fmt;

/// The definitive error type for statement fetching operations.
///
/// When pulling financial data from external sources, things can fail: the network
/// might drop, the credentials might be invalid, or the remote institution could
/// be down. This error encapsulates those failure states so you can gracefully
/// retry or alert the operator.
///
/// ## Examples
///
/// ```
/// use logos_fetch::FetchError;
///
/// let err = FetchError::new("network timeout during statement download");
///
/// // You can extract the error message for the operator
/// assert_eq!(err.to_string(), "network timeout during statement download");
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
