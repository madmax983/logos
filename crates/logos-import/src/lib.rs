//! # The `logos-import` Library
//!
//! Provides utilities for importing data from various external formats.
//!
//! Currently supports importing transactions from simple CSV files,
//! as well as extracting statement records from bank PDFs.

/// Tools for parsing raw text and comma-separated-value records.
pub mod csv;
/// Hashing utilities to detect duplicate transactions.
pub mod fingerprint;
/// Utilities to extract transaction data from unstructured bank PDFs.
pub mod pdf;

pub use csv::{CsvMapping, ImportError, ImportRecord, parse_simple_csv_row};
pub use fingerprint::{deterministic_fingerprint, deterministic_fingerprint_legacy_v1};
pub use pdf::parse_pdf_statement_file;
