//! Logos Fetch
//!
//! This crate provides the capability to fetch financial statements from various institutions.
//! It defines the core abstractions for [`StatementAdapter`]s and configuration models
//! like [`StatementSource`] and [`FetchedStatementArtifact`].

/// Statement adapter definitions and common types for fetching data.
pub(crate) mod adapter;
/// Specific implementations of [`StatementAdapter`](crate::adapter::StatementAdapter).
pub(crate) mod adapters;
/// Configuration models for statement sources.
pub(crate) mod config;
/// Error types for fetch operations.
pub(crate) mod error;
/// Core domain models like `StatementSource` and `FetchRunStatus`.
pub(crate) mod model;
/// Traits and implementations for resolving secrets (e.g., from 1Password).
pub(crate) mod resolver;
/// Domain models for resolved secret bundles.
pub(crate) mod secrets;

pub use adapter::{FakeStatementAdapter, FetchRequest, FetchResult, StatementAdapter};
pub use adapters::ProvidentAdapter;
pub use config::StatementSourceConfig;
pub use error::FetchError;
pub use model::{
    FetchRunStatus, FetchedStatementArtifact, OutputFormat, StatementSource, is_valid_month_key,
};
pub use resolver::{
    OnePasswordCliSecretResolver, OpCliSecretRefReader, SecretRefReader, SecretResolver,
};
pub use secrets::SecretBundle;

/// Indicates whether the crate is ready for use.
#[must_use]
pub const fn crate_ready() -> bool {
    true
}
