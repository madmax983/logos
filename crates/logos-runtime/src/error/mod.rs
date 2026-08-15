//! Runtime error types
use logos_import::ImportError;
use logos_store::StoreError;
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

#[cfg(test)]
mod tests {
    use super::*;
    use logos_core::DomainError;
    use logos_store::StoreError;

    #[test]
    fn test_runtime_error_display() {
        let store_err = RuntimeError::Store(StoreError::PersistFailed {
            message: "disk full".to_string(),
        });
        assert_eq!(store_err.to_string(), "failed to persist store: disk full");

        let import_err = RuntimeError::Import(logos_import::ImportError::InvalidCsvRow {
            message: "bad row".to_string(),
        });
        assert_eq!(import_err.to_string(), "invalid CSV row: bad row");

        let domain_err = RuntimeError::Domain(DomainError::UnbalancedTransaction { total: 100 });
        assert_eq!(
            domain_err.to_string(),
            "domain error: transaction must be balanced to zero, but total was 100"
        );

        let analytics_err = RuntimeError::Analytics {
            message: "not found".to_string(),
        };
        assert_eq!(analytics_err.to_string(), "not found");

        let init_err = RuntimeError::Initialization {
            message: "failed to start".to_string(),
        };
        assert_eq!(init_err.to_string(), "failed to start");
    }

    #[test]
    fn test_runtime_error_from_impls() {
        let store_err = StoreError::PersistFailed {
            message: "test".to_string(),
        };
        let rt_store: RuntimeError = store_err.clone().into();
        match rt_store {
            RuntimeError::Store(e) => assert_eq!(e, store_err),
            _ => panic!("Expected RuntimeError::Store"),
        }

        let import_err = logos_import::ImportError::InvalidCsvRow {
            message: "test".to_string(),
        };
        let rt_import: RuntimeError = import_err.into();
        match rt_import {
            RuntimeError::Import(_) => {}
            _ => panic!("Expected RuntimeError::Import"),
        }

        let domain_err = DomainError::UnbalancedTransaction { total: 100 };
        let rt_domain: RuntimeError = domain_err.clone().into();
        match rt_domain {
            RuntimeError::Domain(e) => assert_eq!(e, domain_err),
            _ => panic!("Expected RuntimeError::Domain"),
        }
    }
}
