//! Traits and implementations for resolving secrets.
//!
//! This module provides the [`SecretResolver`] and [`SecretRefReader`] traits,
//! along with implementations that integrate with the 1Password CLI (`op`).

use std::env;
use std::path::PathBuf;
use std::process::Command;

use crate::{FetchError, SecretBundle, StatementSource};

const LOGOS_FETCH_OP_BIN_ENV: &str = "LOGOS_FETCH_OP_BIN";

/// A trait for resolving secret references from a source into a complete bundle.
/// ## Examples
/// ```ignore
/// impl SecretResolver for MyResolver { ... }
/// ```
pub trait SecretResolver {
    /// Resolves secrets required for a given source.
    ///
    /// # Errors
    /// Returns an error if a secret cannot be read from the underlying secret store.
    fn resolve(&self, source: &StatementSource) -> Result<SecretBundle, FetchError>;
}

/// A trait for retrieving the actual secret value from a reference URI.
/// ## Examples
/// ```ignore
/// impl SecretRefReader for MyReader { ... }
/// ```
pub trait SecretRefReader {
    /// Reads a secret by its reference.
    ///
    /// # Errors
    /// Returns an error if the secret cannot be read.
    fn read_secret_ref(&self, secret_ref: &str) -> Result<String, FetchError>;
}

/// A secret resolver that uses a `SecretRefReader` to retrieve values.
///
/// It delegates the resolution of individual URIs to the inner `R`
/// to create a combined [`SecretBundle`].
#[derive(Debug, Clone)]
pub struct OnePasswordCliSecretResolver<R = OpCliSecretRefReader> {
    reader: R,
}

/// A `SecretRefReader` that invokes the 1Password command-line interface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpCliSecretRefReader {
    op_bin: PathBuf,
}

impl OnePasswordCliSecretResolver<OpCliSecretRefReader> {
    /// Creates a resolver initialized from environment variables.
    ///
    /// This sets up an underlying `OpCliSecretRefReader` configured by the environment.
    ///
    /// # Examples
    ///
    /// ```
    /// use logos_fetch::OnePasswordCliSecretResolver;
    ///
    /// let resolver = OnePasswordCliSecretResolver::from_environment();
    /// ```
    #[must_use]
    pub fn from_environment() -> Self {
        Self {
            reader: OpCliSecretRefReader::from_environment(),
        }
    }
}

impl<R> OnePasswordCliSecretResolver<R> {
    /// Creates a resolver with a specific `SecretRefReader`.
    ///
    /// # Examples
    ///
    /// ```
    /// use logos_fetch::{OnePasswordCliSecretResolver, OpCliSecretRefReader};
    ///
    /// let reader = OpCliSecretRefReader::from_environment();
    /// let resolver = OnePasswordCliSecretResolver::with_reader(reader);
    /// ```
    #[must_use]
    pub const fn with_reader(reader: R) -> Self {
        Self { reader }
    }
}

impl<R> SecretResolver for OnePasswordCliSecretResolver<R>
where
    R: SecretRefReader,
{
    fn resolve(&self, source: &StatementSource) -> Result<SecretBundle, FetchError> {
        let username = self.reader.read_secret_ref(source.username_secret_ref())?;
        let password = self.reader.read_secret_ref(source.password_secret_ref())?;
        let totp_code = source
            .totp_secret_ref()
            .map(|secret_ref| self.reader.read_secret_ref(secret_ref))
            .transpose()?;

        SecretBundle::new(
            username.trim(),
            password.trim(),
            totp_code.as_deref().map(str::trim),
        )
    }
}

impl OpCliSecretRefReader {
    /// Configures the reader with a specific `op` binary path.
    #[must_use]
    pub const fn new(op_bin: PathBuf) -> Self {
        Self { op_bin }
    }

    /// Configures the reader using environment variables.
    ///
    /// It looks for the `LOGOS_FETCH_OP_BIN` environment variable to locate
    /// the 1Password executable. If absent, it defaults to `"op"`.
    ///
    /// # Examples
    ///
    /// ```
    /// use logos_fetch::OpCliSecretRefReader;
    ///
    /// let reader = OpCliSecretRefReader::from_environment();
    /// ```
    #[must_use]
    pub fn from_environment() -> Self {
        let op_bin =
            env::var_os(LOGOS_FETCH_OP_BIN_ENV).map_or_else(|| PathBuf::from("op"), PathBuf::from);
        Self { op_bin }
    }
}

impl SecretRefReader for OpCliSecretRefReader {
    fn read_secret_ref(&self, secret_ref: &str) -> Result<String, FetchError> {
        let output = Command::new(&self.op_bin)
            .arg("read")
            .arg(secret_ref)
            .output()
            .map_err(|err| {
                FetchError::new(format!(
                    "failed to execute 1Password CLI '{} read {}': {err}",
                    self.op_bin.display(),
                    secret_ref
                ))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
            let summary = if stderr.is_empty() {
                format!("exit status {}", output.status)
            } else {
                stderr
            };
            return Err(FetchError::new(format!(
                "1Password CLI could not read '{secret_ref}': {summary}"
            )));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}
