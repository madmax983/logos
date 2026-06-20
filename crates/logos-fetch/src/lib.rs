//! # The `logos-fetch` Library
//!
//! Welcome to the retrieval engine of the `logos` accounting system.
//! While `logos-core` deals with strict financial truths, `logos-fetch` handles
//! the messy reality of the outside world: securely resolving credentials, communicating
//! with external financial institutions, and downloading statement artifacts (PDF/CSV)
//! so they can eventually be parsed and imported.
//!
//! ## The Pipeline
//!
//! 1. **Configuration**: Define what you want to fetch using a [`StatementSourceConfig`].
//!    This tells the system *where* to get data and *where* your credentials live.
//! 2. **Resolution**: The system uses a [`SecretResolver`] (like 1Password) to turn those
//!    secret references into an actual [`SecretBundle`].
//! 3. **Execution**: A [`StatementAdapter`] takes the [`FetchRequest`] and the `SecretBundle`
//!    to actually perform the download, yielding a [`FetchedStatementArtifact`].
//!
//! ## Examples
//!
//! Here is a simplified example of the fetch pipeline using fake adapters.
//!
//! ```
//! use logos_fetch::{
//!     FakeStatementAdapter, FetchRequest, OutputFormat, SecretBundle,
//!     StatementAdapter, StatementSource,
//! };
//!
//! # tokio::runtime::Builder::new_current_thread().build().unwrap().block_on(async {
//! // 1. Configure the source
//! let source = StatementSource::new(
//!     "test_bank",
//!     "bank",
//!     "assets:checking",
//!     vec![OutputFormat::Pdf]
//! ).unwrap();
//!
//! // 2. Resolve credentials (we'll just create a bundle directly here for the example)
//! let secrets = SecretBundle::new("user", "pass", None).unwrap();
//!
//! // 3. Set up the request and adapter
//! let request = FetchRequest::new(&source, "2026-02").unwrap();
//! let adapter = FakeStatementAdapter::download_fixture_statement();
//!
//! // 4. Execute the fetch!
//! let result = adapter.fetch(&request, &secrets).await.unwrap();
//! assert!(result.status().is_success());
//! # });
//! ```

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
pub use model::{FetchRunStatus, FetchedStatementArtifact, OutputFormat, StatementSource};
pub use resolver::{
    OnePasswordCliSecretResolver, OpCliSecretRefReader, SecretRefReader, SecretResolver,
};
pub use secrets::SecretBundle;

/// Indicates whether the crate is ready for use.
#[must_use]
pub const fn crate_ready() -> bool {
    true
}
