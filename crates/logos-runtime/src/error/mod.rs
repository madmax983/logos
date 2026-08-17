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
    use logos_import::ImportError;
    use logos_store::StoreError;

    #[test]
    fn test_runtime_error_display() {
        let store_err = RuntimeError::Store(StoreError::ConnectionFailed {
            message: "store err".into(),
        });
        assert_eq!(store_err.to_string(), "store err");

        let import_err = RuntimeError::Import(ImportError::InvalidAmount);
        assert_eq!(import_err.to_string(), "invalid amount in CSV row");

        let domain_err = RuntimeError::Domain(DomainError::EmptyAccountId);
        assert_eq!(
            domain_err.to_string(),
            "domain error: account id cannot be empty"
        );

        let analytics_err = RuntimeError::Analytics {
            message: "analytics err".into(),
        };
        assert_eq!(analytics_err.to_string(), "analytics err");

        let init_err = RuntimeError::Initialization {
            message: "init err".into(),
        };
        assert_eq!(init_err.to_string(), "init err");
    }

    #[test]
    fn test_runtime_error_from() {
        let store_err = StoreError::ConnectionFailed {
            message: "store err".into(),
        };
        assert!(matches!(
            RuntimeError::from(store_err),
            RuntimeError::Store(_)
        ));

        let import_err = ImportError::InvalidAmount;
        assert!(matches!(
            RuntimeError::from(import_err),
            RuntimeError::Import(_)
        ));

        let domain_err = DomainError::EmptyAccountId;
        assert!(matches!(
            RuntimeError::from(domain_err),
            RuntimeError::Domain(_)
        ));
    }
}
