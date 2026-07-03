//! Models for resolved secrets used during a fetch run.
//!
//! This module defines the [`SecretBundle`] which holds the actual
//! sensitive credentials after they have been successfully resolved
//! from an external store (like 1Password).

use crate::FetchError;

/// A securely resolved bundle of credentials for a specific fetch operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecretBundle {
    /// The resolved username.
    username: String,
    /// The resolved password.
    password: String,
    /// The resolved Time-Based One-Time Password, if required.
    totp_code: Option<String>,
}

impl SecretBundle {
    /// Creates a runtime bundle of resolved credentials.
    ///
    /// # Examples
    ///
    /// ```
    /// use logos_fetch::SecretBundle;
    ///
    /// let bundle = SecretBundle::new("user_123", "hunter2", Some("123456")).unwrap();
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error when username or password is empty.
    pub fn new(
        username: &str,
        password: &str,
        totp_code: Option<&str>,
    ) -> Result<Self, FetchError> {
        if username.trim().is_empty() {
            return Err(FetchError::new("secret username must not be empty"));
        }
        if password.trim().is_empty() {
            return Err(FetchError::new("secret password must not be empty"));
        }

        Ok(Self {
            username: username.trim().to_owned(),
            password: password.trim().to_owned(),
            totp_code: totp_code.map(str::to_owned),
        })
    }

    /// Returns the resolved plaintext username to be used for the fetch.
    ///
    /// This value is passed securely directly to the adapter execution environment.
    ///
    /// ## Examples
    ///
    /// ```
    /// use logos_fetch::SecretBundle;
    /// let bundle = SecretBundle::new("alice", "pass123", None).unwrap();
    /// assert_eq!(bundle.username(), "alice");
    /// ```
    #[must_use]
    pub fn username(&self) -> &str {
        &self.username
    }

    /// Returns the resolved plaintext password to be used for the fetch.
    ///
    /// This value is passed securely directly to the adapter execution environment.
    ///
    /// ## Examples
    ///
    /// ```
    /// use logos_fetch::SecretBundle;
    /// let bundle = SecretBundle::new("alice", "pass123", None).unwrap();
    /// assert_eq!(bundle.password(), "pass123");
    /// ```
    #[must_use]
    pub fn password(&self) -> &str {
        &self.password
    }

    /// Returns the active, resolved TOTP code to pass into multi-factor auth challenges.
    ///
    /// This will be `None` if the statement source was not configured with a TOTP secret ref.
    ///
    /// ## Examples
    ///
    /// ```
    /// use logos_fetch::SecretBundle;
    /// let bundle = SecretBundle::new("alice", "pass123", Some("456789")).unwrap();
    /// assert_eq!(bundle.totp_code(), Some("456789"));
    /// ```
    #[must_use]
    pub fn totp_code(&self) -> Option<&str> {
        self.totp_code.as_deref()
    }
}
