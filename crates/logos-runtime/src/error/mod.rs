//! Runtime error types
use logos_import::ImportError;
use logos_store::StoreError;
#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    #[error(transparent)]
    Store(#[from] StoreError),
    #[error(transparent)]
    Import(#[from] ImportError),
    #[error("domain error: {0}")]
    Domain(#[from] logos_core::DomainError),
    #[error("{message}")]
    Analytics { message: String },
    #[error("{message}")]
    Initialization { message: String },
}
