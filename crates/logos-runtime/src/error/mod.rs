#![allow(missing_docs)]
use logos_import::ImportError;
use logos_store::error::StoreError;
use std::fmt;

#[derive(Debug)]
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
