//! The Logos Application Runtime
//!
//! This module serves as the primary coordination layer for the Logos application.
//! It brings together storage (`logos-store`), domain logic (`logos-core`), statement fetching (`logos-fetch`),
//! data ingestion (`logos-import`), and high-level reporting (`logos-reporting`).
//!
//! # The Runtime Core
//!
//! The [`AppRuntime`] orchestrates higher-order actions like "running the month autopilot" or
//! "generating a system-wide analytics snapshot." It holds the necessary configurations (like the fetch
//! config path) and a generic reference to a storage backend (implementing `LedgerStore`).
//!
//! ## Examples
//!
//! ```
//! use logos_runtime::{AppRuntime, MonthAutopilotRequest};
//! use logos_store::MemoryStore;
//!
//! // 1. Start a new runtime with an in-memory store
//! let mut runtime = AppRuntime::<MemoryStore>::new_in_memory();
//!
//! // 2. Post a manual transaction
//! let tx_id = runtime.post_double_entry("Initial Deposit", "assets:checking", "income:salary", 5000_00).unwrap();
//! assert!(runtime.transaction_exists(&tx_id));
//!
//! // 3. Run high-level reports across the entire ledger
//! let balance = runtime.register_balance_for("assets:checking");
//! assert_eq!(balance, 5000_00);
//! ```

pub(crate) mod error;
pub(crate) mod models;
pub(crate) mod runtime;

pub use error::RuntimeError;
pub use models::*;
pub use runtime::{
    AppRuntime, default_artifacts_root, default_fetch_config_path, default_state_root,
};
