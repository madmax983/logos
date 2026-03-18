//! Concrete statement adapter implementations.
//!
//! This module contains specific adapters capable of interacting with
//! different external tools or APIs to fetch financial statements.

/// The Provident wrapper adapter.
pub mod provident;

pub use provident::ProvidentAdapter;
