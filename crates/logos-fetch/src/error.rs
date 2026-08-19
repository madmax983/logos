//! Error handling for statement fetching.
//!
//! This module defines the central [`FetchError`] type used throughout the `logos-fetch` crate
//! to represent failures encountered when configuring sources, resolving credentials,
//! or executing statement adapter downloads.

use core::fmt;

/// A domain error indicating a failure during a statement fetch operation.
///
/// This is an opaque error type that simply wraps an internal message string,
/// suitable for displaying to an operator or logging.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchError {
    message: String,
}

impl FetchError {
    /// Creates a new fetch error with the given message.
    ///
    /// # Examples
    ///
    /// ```
    /// use logos_fetch::FetchError;
    ///
    /// let error = FetchError::new("network timeout during statement download");
    /// assert_eq!(error.to_string(), "network timeout during statement download");
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
