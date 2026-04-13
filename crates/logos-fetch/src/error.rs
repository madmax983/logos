use core::fmt;

/// A generic error type representing a failure during the automated fetching process.
///
/// This wraps underlying errors from headless browsers, network timeouts, or invalid
/// configuration into a single opaque error type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchError {
    message: String,
}

impl FetchError {
    /// Constructs a new `FetchError` with the provided string message.
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
