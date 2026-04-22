//! # The `logos-core` Library
//!
//! Welcome to the beating heart of `logos`. This library implements a strict,
//! zero-trust double-entry accounting engine and sophisticated financial planning tools.
//!
//! It forces users to construct valid, fully balanced transactions where debits (+ve) and
//! credits (-ve) perfectly net out. It also refuses to build objects without names,
//! bounds, or logical invariants satisfied.
//!
//! ## Modules
//!
//! - **[`domain`]**: The foundational pieces of the ledger. Accounts, Transactions, Budgets, and RSUs.
//! - **[`error`]**: Defines [`DomainError`], the one-stop shop for everything that can go wrong when breaking the rules.
//! - **[`planning`]**: High-level financial forecasting. From automated RSU distribution to FIRE simulations and Net Worth Projection.
//! - **[`experimental`]**: Beta features or proofs of concept. Use at your own risk.
//!
//! ## Examples
//!
//! The core library ensures that you cannot construct an invalid financial state.
//! Here is a quick example of defining accounts and moving money between them securely.
//!
//! ```
//! use logos_core::{AccountId, DomainError, Posting, TransactionBuilder};
//!
//! // 1. Define your accounts. The type system prevents empty names.
//! let checking = AccountId::new("assets:checking").unwrap();
//! let salary = AccountId::new("income:salary").unwrap();
//!
//! // 2. Construct a transaction. The builder enforces double-entry rules.
//! let txn = TransactionBuilder::new("March Salary")
//!     .posting(Posting::debit(checking.clone(), 5000_00).unwrap())
//!     .posting(Posting::credit(salary.clone(), 5000_00).unwrap())
//!     .build()
//!     .expect("This transaction balances perfectly!");
//!
//! assert_eq!(txn.postings().len(), 2);
//!
//! // 3. What happens if we mess up? The engine rejects it immediately.
//! let bad_txn = TransactionBuilder::new("Oops")
//!     .posting(Posting::debit(checking, 100_00).unwrap())
//!     // Forgot the credit!
//!     .build();
//!
//! assert!(matches!(bad_txn, Err(DomainError::UnbalancedTransaction { total: 100_00 })));
//! ```

pub(crate) mod domain;
pub(crate) mod error;
/// Experimental features and proof-of-concepts. Use at your own risk.
pub mod experimental;
pub mod planning;

pub use domain::account::{AccountId, AccountType};
pub use domain::budget::{BudgetMonth, rollover_end_balance};
pub use domain::category::{Category, CategoryGroup, CategoryGroupId};
pub use domain::correction::{Correction, TransactionId};
pub use domain::rsu::{AllocationPolicy, HaircutTierTable, forecast_value_cents};
pub use domain::transaction::{Posting, Transaction, TransactionBuilder};
pub use error::DomainError;
