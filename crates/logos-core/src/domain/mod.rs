//! # The Domain Core
//!
//! This module houses the foundational building blocks of the `logos` accounting system.
//! Here you will find the absolute truths of the ledger—the strict double-entry invariants,
//! the envelope budgeting mechanics, and the forward-looking RSU forecasting models.
//!
//! Every module here is designed to enforce business rules at construction time,
//! meaning that invalid financial states (like unbalanced transactions or negative equity forecasts)
//! are structurally impossible to create.

pub mod account;
pub mod budget;
pub mod category;
pub mod correction;
pub mod rsu;
pub mod transaction;
