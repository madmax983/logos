//! Error types for statement fetching.
//!
//! This module defines the [`FetchError`] type, which is used to indicate
//! fatal failures during configuration parsing, secret resolution, or
//! runner execution. Unlike [`FetchRunStatus`](crate::FetchRunStatus) which
//! describes the outcome of a successful execution attempt, a `FetchError`
//! indicates that the process could not be completed at all.

use core::fmt;

/// An error that occurred during a fetch operation.
///
/// Operators should capture these errors and log them, as they typically
/// represent system misconfigurations (e.g., missing credentials, invalid TOML)
/// rather than expected runtime states (like requiring an SMS challenge).
///
/// ## Examples
///
/// ```
/// use logos_fetch::FetchError;
///
/// let error = FetchError::new("invalid configuration file");
/// assert_eq!(error.to_string(), "invalid configuration file");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchError {
    message: String,
}

impl FetchError {
    /// Creates a new fetch error with the provided message.
    ///
    /// ## Examples
    ///
    /// ```
    /// use logos_fetch::FetchError;
    ///
    /// let err = FetchError::new("missing username secret ref");
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
