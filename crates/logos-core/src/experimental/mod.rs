//! Experimental modeling tools and financial projections.
//!
//! This module contains features that build on top of the core double-entry domain
//! to provide higher-level insights or automated workflows. These features are
//! considered "experimental" as their APIs may evolve faster than the foundational
//! ledger types.
//!
//! - The [`fire`] module provides tools for calculating Financial Independence / Retire Early metrics.
//! - The [`rsu_distributor`] module provides a mechanism to automatically construct balanced
//!   transactions that distribute unvested equity according to a target allocation policy.

pub mod fire;
pub mod rsu_distributor;
