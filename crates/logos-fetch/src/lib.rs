pub mod adapter;
pub mod adapters;
pub mod config;
pub mod error;
pub mod model;
pub mod resolver;
pub mod secrets;

pub use adapter::{FakeStatementAdapter, FetchRequest, FetchResult, StatementAdapter};
pub use adapters::ProvidentAdapter;
pub use config::StatementSourceConfig;
pub use error::FetchError;
pub use model::{FetchRunStatus, FetchedStatementArtifact, OutputFormat, StatementSource};
pub use resolver::{
    OnePasswordCliSecretResolver, OpCliSecretRefReader, SecretRefReader, SecretResolver,
};
pub use secrets::SecretBundle;

#[must_use]
pub const fn crate_ready() -> bool {
    true
}
