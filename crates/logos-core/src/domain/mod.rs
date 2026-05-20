//! # The Domain Core
//!
//! This module houses the foundational building blocks of the `logos` accounting system.
//! Here you will find the absolute truths of the ledger—the strict double-entry invariants,
//! the envelope budgeting mechanics, and the forward-looking RSU forecasting models.
//!
//! Every module here is designed to enforce business rules at construction time,
//! meaning that invalid financial states (like unbalanced transactions or negative equity forecasts)
//! are structurally impossible to create.

#[allow(clippy::redundant_pub_crate)]
pub(crate) mod account;
#[allow(clippy::redundant_pub_crate)]
pub(crate) mod budget;
#[allow(clippy::redundant_pub_crate)]
pub(crate) mod category;
#[allow(clippy::redundant_pub_crate)]
pub(crate) mod correction;
#[allow(clippy::redundant_pub_crate)]
pub(crate) mod rsu;
#[allow(clippy::redundant_pub_crate)]
pub(crate) mod transaction;
