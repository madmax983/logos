//! Error types for the fetch execution process.
//!
//! This module provides the central error type used throughout the `logos-fetch`
//! crate. It captures failures that occur when configuring sources, resolving secrets,
//! or executing the actual statement fetch operation.

use core::fmt;

/// An error that occurred during a statement fetch operation.
///
/// `FetchError` is a simple string-based error type that describes what went wrong
/// when attempting to download a financial statement or parse its metadata.
///
/// ## Examples
///
/// ```
/// use logos_fetch::FetchError;
///
/// let error = FetchError::new("failed to connect to bank server");
/// assert_eq!(error.to_string(), "failed to connect to bank server");
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
