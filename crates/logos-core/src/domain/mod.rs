//! # The Domain Core
//!
//! This module houses the foundational building blocks of the `logos` accounting system.
//! Here you will find the absolute truths of the ledger—the strict double-entry invariants,
//! the envelope budgeting mechanics, and the forward-looking RSU forecasting models.
//!
//! Every module here is designed to enforce business rules at construction time,
//! meaning that invalid financial states (like unbalanced transactions or negative equity forecasts)
//! are structurally impossible to create.
//!
//! ## Core Modules
//!
//! - **[`account`]**: Account types and their double-entry behaviors.
//! - **[`budget`]**: Envelope budgeting and rollover calculations.
//! - **[`category`]**: Budget categories and grouping logic.
//! - **[`correction`]**: The immutable ledger's correction mechanism.
//! - **[`rsu`]**: Restricted Stock Unit forecasting and allocation.
//! - **[`transaction`]**: Strictly balanced double-entry transactions.

pub mod account;
pub mod budget;
pub mod category;
pub mod correction;
pub mod rsu;
pub mod transaction;
