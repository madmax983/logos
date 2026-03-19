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

/// Foundational domain models and logic for double-entry accounting.
pub mod domain;
/// Error definitions mapping to domain rule violations.
pub mod error;
/// Unstable/beta features and utilities.
pub mod experimental;
/// High-level modules for financial planning and forecasting.
pub mod planning;

pub use domain::account::{AccountId, AccountType};
pub use domain::budget::{BudgetMonth, rollover_end_balance};
pub use domain::category::{Category, CategoryGroup, CategoryGroupId};
pub use domain::correction::{Correction, TransactionId};
pub use domain::rsu::{AllocationPolicy, HaircutTierTable, forecast_value_cents};
pub use domain::transaction::{Posting, Transaction, TransactionBuilder};
pub use error::DomainError;
