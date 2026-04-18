//! # The `logos-store` Persistence Contract
//!
//! Welcome to the Storage Layer. This module defines the boundaries of how financial data
//! is saved and loaded, without tying the system to any specific database engine.
//!
//! The central piece is the [`LedgerStore`] trait. It enforces the rules of interaction:
//! appending transactions, fetching account balances, and recording reconciliation events.
//!
//! By abstracting storage behind a trait, we enable:
//! 1. **Testing:** A blazing-fast [`MemoryStore`] implementation that allows the rest of the
//!    application to be tested without spinning up Postgres.
//! 2. **Safety:** Strict boundaries ensure that higher-level logic cannot bypass business
//!    rules to execute raw SQL queries.
//!
//! ## Examples
//!
//! ```
//! use logos_store::{MemoryStore, LedgerStore};
//!
//! // The MemoryStore implements LedgerStore, making it perfect for tests.
//! let mut store = MemoryStore::default();
//!
//! // Ensure the method correctly compiles based on actual available traits.
//! let count = store.fetch_run_count();
//! assert_eq!(count, 0);
//! ```

pub mod error;
pub mod memory;
pub mod model;
pub mod traits;

pub use error::StoreError;
pub use memory::MemoryStore;
pub use model::*;
pub use traits::LedgerStore;
