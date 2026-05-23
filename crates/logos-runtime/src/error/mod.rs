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

    #[test]
    fn should_display_store_error() {
        let err = RuntimeError::Store(StoreError::ConnectionFailed {
            message: "timeout".to_string(),
        });
        assert_eq!(err.to_string(), "timeout");
    }

    #[test]
    fn should_display_domain_error() {
        let domain_err = logos_core::DomainError::EmptyAccountId;
        let expected = format!("domain error: {domain_err}");
        let err = RuntimeError::Domain(domain_err);
        assert_eq!(err.to_string(), expected);
    }

    #[test]
    fn should_display_analytics_error() {
        let err = RuntimeError::Analytics {
            message: "no data".to_string(),
        };
        assert_eq!(err.to_string(), "no data");
    }

    #[test]
    fn should_display_initialization_error() {
        let err = RuntimeError::Initialization {
            message: "config missing".to_string(),
        };
        assert_eq!(err.to_string(), "config missing");
    }

    #[test]
    fn should_convert_into_runtime_error() {
        let store_err = StoreError::ConnectionFailed {
            message: "timeout".to_string(),
        };
        let err: RuntimeError = store_err.into();
        assert!(matches!(err, RuntimeError::Store(_)));

        let domain_err = logos_core::DomainError::EmptyAccountId;
        let err: RuntimeError = domain_err.into();
        assert!(matches!(err, RuntimeError::Domain(_)));
    }
}
