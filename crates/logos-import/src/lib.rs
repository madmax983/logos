//! The Gateway to the Zero-Trust Ledger.
//!
//! `logos-import` bridges the gap between messy, unpredictable external world data (Bank CSVs and PDFs)
//! and the strict, zero-trust rules of `logos-core`.
//!
//! Its primary responsibilities are:
//! 1. **Normalization:** Converting varied bank formats into a uniform [`ImportRecord`].
//! 2. **Idempotency:** Generating a cryptographically secure, deterministic fingerprint using Blake3
//!    to ensure that overlapping statement exports never result in double-counting transactions.
//!
//! ## Examples
//!
//! ```
//! use logos_import::{CsvMapping, parse_simple_csv_row, deterministic_fingerprint};
//!
//! // 1. Define how to read a specific bank's CSV format
//! let mapping = CsvMapping {
//!     source_id: "chase_checking.csv".to_string(),
//!     timestamp_idx: 0,
//!     amount_idx: 2,
//!     memo_idx: 1,
//!     account_idx: 3,
//!     category_idx: 4,
//! };
//!
//! // 2. Parse a messy row into a clean ImportRecord
//! let raw_csv = "2023-10-01,TARGET STORE,-5042,assets:checking,expenses:groceries";
//! let record = parse_simple_csv_row(raw_csv, &mapping).unwrap();
//!
//! // 3. Generate an idempotent fingerprint to prevent double-importing
//! let fingerprint = deterministic_fingerprint(&record);
//!
//! assert_eq!(record.amount_cents(), -5042);
//! assert_eq!(record.memo(), "TARGET STORE");
//! ```
pub mod csv;
pub mod fingerprint;
pub mod pdf;

pub use csv::{CsvMapping, ImportError, ImportRecord, parse_simple_csv_row};
pub use fingerprint::{deterministic_fingerprint, deterministic_fingerprint_legacy_v1};
pub use pdf::parse_pdf_statement_file;
