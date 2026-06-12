use core::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
/// Represents an error that occurred while fetching statements from an external source.
///
/// This error acts as a boundary type for the `logos-fetch` crate, encapsulating
/// failures from network requests, authentication issues, or parsing problems.
///
/// ## Examples
///
/// ```
/// use logos_fetch::FetchError;
///
/// let err = FetchError::new("failed to connect to bank api");
/// assert_eq!(err.to_string(), "failed to connect to bank api");
/// ```
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
