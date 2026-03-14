use std::env;
use std::path::PathBuf;
use std::process::Command;

use crate::{FetchError, SecretBundle, StatementSource};

const LOGOS_FETCH_OP_BIN_ENV: &str = "LOGOS_FETCH_OP_BIN";

pub trait SecretResolver {
    /// Resolves secrets required for a given statement source.
    ///
    /// # Errors
    ///
    /// Returns a `FetchError` if any required secret cannot be resolved.
    fn resolve(&self, source: &StatementSource) -> Result<SecretBundle, FetchError>;
}

pub trait SecretRefReader {
    /// Reads a secret using its reference.
    ///
    /// # Errors
    ///
    /// Returns a `FetchError` if reading the secret fails.
    fn read_secret_ref(&self, secret_ref: &str) -> Result<String, FetchError>;
}

#[derive(Debug, Clone)]
pub struct OnePasswordCliSecretResolver<R = OpCliSecretRefReader> {
    reader: R,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpCliSecretRefReader {
    op_bin: PathBuf,
}

impl OnePasswordCliSecretResolver<OpCliSecretRefReader> {
    #[must_use]
    pub fn from_environment() -> Self {
        Self {
            reader: OpCliSecretRefReader::from_environment(),
        }
    }
}

impl<R> OnePasswordCliSecretResolver<R> {
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
