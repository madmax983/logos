//! # The `logos-runtime` Engine
//!
//! Welcome to the Orchestration Layer. If `logos-core` is the brain containing the pure
//! mathematical rules of double-entry accounting, `logos-runtime` is the body that interacts
//! with the physical world.
//!
//! This module solves the "Black Box" of integration. It defines the [`AppRuntime`], which
//! glues together:
//! - **Persistence:** Reading and writing ledgers via `logos-store`.
//! - **Network:** Fetching bank statements automatically via `logos-fetch`.
//! - **Ingestion:** Parsing and fingerprinting raw CSV/PDFs via `logos-import`.
//!
//! By keeping these impure side-effects outside of `logos-core`, we maintain strict boundaries
//! where business rules remain deterministic and fully testable.
//!
//! ## Examples
//!
//! ```
//! use logos_runtime::AppRuntime;
//! use logos_store::MemoryStore;
//! use std::path::PathBuf;
//!
//! // 1. Set up an ephemeral in-memory storage backend for testing.
//! let store = MemoryStore::default();
//!
//! // 2. Create the orchestration runtime, telling it where to save analytics artifacts.
//! let runtime = AppRuntime::with_store(store, PathBuf::from("/tmp/logos-artifacts"), None);
//! ```

pub mod error;
pub mod models;
pub mod runtime;

pub use error::RuntimeError;
pub use models::*;
pub use runtime::AppRuntime;
