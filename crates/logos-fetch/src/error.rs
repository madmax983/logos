//! Error types for the `logos-fetch` crate.
//!
//! This module defines the [`FetchError`] type, which represents failures
//! that occur during statement configuration parsing, secret resolution,
//! or adapter execution. When a fetch operation fails, operators can use the
//! message inside the error to diagnose whether the issue is a missing 1Password
//! secret, a malformed TOML file, or an institution timeout.

use core::fmt;

/// The overarching error type for statement fetching operations.
///
/// This error is returned when a fetch request fails due to configuration
/// issues, network problems, or adapter-specific errors.
///
/// # Examples
///
/// ```
/// use logos_fetch::FetchError;
///
/// let error = FetchError::new("1Password CLI timeout");
/// assert_eq!(error.to_string(), "1Password CLI timeout");
/// ```
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
    /// let error = FetchError::new("adapter offline");
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
