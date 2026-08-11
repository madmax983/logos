use core::fmt;

/// An error that occurs during a statement fetch operation.
///
/// This error captures failures related to parsing the statement source configuration,
/// resolving secrets from an external store (like 1Password), or executing the underlying
/// fetch adapter.
///
/// # Examples
///
/// ```
/// use logos_fetch::FetchError;
///
/// // Create an error representing a missing environment variable or missing secret
/// let err = FetchError::new("1Password CLI could not resolve 'op://vault/item'");
/// assert_eq!(err.to_string(), "1Password CLI could not resolve 'op://vault/item'");
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
