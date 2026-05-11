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
    fn test_runtime_error_formatting() {
        let err = RuntimeError::Analytics {
            message: "test".to_string(),
        };
        assert_eq!(err.to_string(), "test");

        let err = RuntimeError::Initialization {
            message: "init failed".to_string(),
        };
        assert_eq!(err.to_string(), "init failed");

        let err = RuntimeError::Domain(DomainError::EmptyAccountId);
        assert_eq!(
            err.to_string(),
            format!("domain error: {}", DomainError::EmptyAccountId)
        );

        let err = RuntimeError::Store(StoreError::Domain(DomainError::EmptyAccountId));
        assert_eq!(
            err.to_string(),
            format!("{}", StoreError::Domain(DomainError::EmptyAccountId))
        );

        let err = RuntimeError::Import(ImportError::InvalidAmount);
        assert_eq!(err.to_string(), format!("{}", ImportError::InvalidAmount));
    }

    #[test]
    fn test_runtime_error_from_conversions() {
        let store_err = StoreError::Domain(DomainError::EmptyAccountId);
        let err: RuntimeError = store_err.into();
        assert!(matches!(err, RuntimeError::Store(_)));

        let import_err = ImportError::InvalidAmount;
        let err: RuntimeError = import_err.into();
        assert!(matches!(err, RuntimeError::Import(_)));

        let domain_err = DomainError::EmptyAccountId;
        let err: RuntimeError = domain_err.into();
        assert!(matches!(err, RuntimeError::Domain(_)));
    }
}
