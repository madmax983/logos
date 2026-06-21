use core::fmt;

/// An error that occurred while attempting to fetch a statement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchError {
    message: String,
}

impl FetchError {
    /// Creates a new fetch error with the provided message.
    ///
    /// # Examples
    ///
    /// ```
    /// use logos_fetch::FetchError;
    ///
    /// let err = FetchError::new("network timeout");
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
