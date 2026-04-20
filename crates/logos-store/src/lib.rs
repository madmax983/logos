//! # The `logos-store` Library
//!
//! The Persistence Contract. This crate acts as the boundary between the pure business logic in `logos-core`
//! and the harsh reality of databases and file systems.
//!
//! By defining the [`LedgerStore`] trait here, `logos-core` doesn't need to know if data is saved in `PostgreSQL`,
//! a local file, or just memory. It only knows that the storage contract will be upheld.
//!
//! ## The "Black Box" of Storage
//!
//! You interact with storage via the `LedgerStore` trait. This crate provides a [`MemoryStore`] implementation
//! for rapid testing, while production uses implementations like `PostgresStore` (found in `logos-store-pg`).
//!
//! ## Examples
//!
//! ```
//! use logos_store::{LedgerStore, MemoryStore};
//! use logos_core::{TransactionBuilder, Posting, AccountId};
//!
//! // 1. Instantiate the in-memory storage for testing
//! let mut store = MemoryStore::new();
//!
//! // 2. Ensure it starts clean
//! assert_eq!(store.transaction_count(), 0);
//!
//! // 3. Build a valid transaction
//! let checking = AccountId::new("assets:checking").unwrap();
//! let salary = AccountId::new("income:salary").unwrap();
//! let txn = TransactionBuilder::new("Payday")
//!     .posting(Posting::debit(checking, 1000_00).unwrap())
//!     .posting(Posting::credit(salary, 1000_00).unwrap());
//!
//! // 4. Persist it! The store now safely holds the ledger entry.
//! let _id = store.write_transaction(txn).unwrap();
//! assert_eq!(store.transaction_count(), 1);
//! ```

pub mod error;
pub mod memory;
pub mod model;
pub mod traits;

pub use error::StoreError;
pub use memory::MemoryStore;
pub use model::*;
pub use traits::LedgerStore;
