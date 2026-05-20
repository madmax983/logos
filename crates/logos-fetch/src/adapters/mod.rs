//! Concrete statement adapter implementations.
//!
//! This module contains specific adapters capable of interacting with
//! different external tools or APIs to fetch financial statements.

/// The Provident wrapper adapter.
#[allow(clippy::redundant_pub_crate)]
pub(crate) mod provident;

pub use provident::ProvidentAdapter;
