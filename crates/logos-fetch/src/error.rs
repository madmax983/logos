use core::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
/// A failure occurring during the execution of a statement fetch or artifact parsing.
///
/// Fetch operations rely on external credentials, local configuration, and network
/// calls. This error type consolidates failures from the 1Password CLI resolver,
/// file I/O during adapter runner execution, and internal validation.
///
/// ## Examples
///
/// ```
/// use logos_fetch::FetchError;
///
/// let err = FetchError::new("1Password CLI could not read \'op://vault/item\'");
/// assert_eq!(err.to_string(), "1Password CLI could not read \'op://vault/item\'");
/// ```
pub struct FetchError {
    message: String,
}

impl FetchError {
    /// Wraps an operator-facing message detailing the cause of the failure.
    ///
    /// ## Examples
    ///
    /// ```
    /// use logos_fetch::FetchError;
    /// let err = FetchError::new("Invalid credentials");
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
