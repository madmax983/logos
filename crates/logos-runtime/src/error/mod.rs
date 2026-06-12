//! The Unified Error Boundary
//!
//! This module defines the [`RuntimeError`] enum, which consolidates all domain, storage, and
//! import errors into a single, unified type used throughout the application's runtime orchestration.
//! It ensures that failures bubble up cleanly without forcing the caller to handle disparate error types.

use logos_import::ImportError;
use logos_store::StoreError;
use std::fmt;

#[derive(Debug)]
/// The unified error type for all runtime operations.
///
/// Consolidates underlying failures from storage, domain logic, and analytics generation.
///
/// ## Examples
///
/// ```
/// use logos_runtime::RuntimeError;
/// use logos_core::DomainError;
///
/// let domain_err = DomainError::EmptyAccountId;
/// let runtime_err: RuntimeError = domain_err.into();
///
/// // The error transparently wraps the underlying domain error string
/// let error_string = runtime_err.to_string();
/// assert!(error_string.starts_with("domain error:"));
/// ```
pub enum RuntimeError {
    Store(StoreError),
    Import(ImportError),
    Domain(logos_core::DomainError),
    Analytics { message: String },
    Initialization { message: String },
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Store(err) => write!(f, "{err}"),
            Self::Import(err) => write!(f, "{err}"),
            Self::Domain(err) => write!(f, "domain error: {err}"),
            Self::Analytics { message } | Self::Initialization { message } => {
                write!(f, "{message}")
            }
        }
    }
}

impl std::error::Error for RuntimeError {}

impl From<StoreError> for RuntimeError {
    fn from(value: StoreError) -> Self {
        Self::Store(value)
    }
}

impl From<ImportError> for RuntimeError {
    fn from(value: ImportError) -> Self {
        Self::Import(value)
    }
}

impl From<logos_core::DomainError> for RuntimeError {
    fn from(value: logos_core::DomainError) -> Self {
        Self::Domain(value)
    }
}
