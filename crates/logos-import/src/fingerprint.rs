//! Deterministic fingerprint generation.
//!
//! Provides cryptographic, deterministic hashing of imported statements to prevent
//! duplicate ingestion. When `logos` imports records from CSV or PDF, it generates
//! a unique fingerprint based on the *semantic content* of the record (timestamp,
//! amount, memo, etc). If a fingerprint already exists in the ledger, the row is
//! safely ignored as a duplicate.

use std::hash::{Hash, Hasher as StdHasher};

use blake3::Hasher;

use crate::csv::ImportRecord;

const FINGERPRINT_V2_DOMAIN: &[u8] = b"logos-import:fingerprint:v2";
const FIELD_SOURCE_ID: u8 = 1;
const FIELD_TIMESTAMP: u8 = 2;
const FIELD_AMOUNT_CENTS: u8 = 3;
const FIELD_MEMO: u8 = 4;
const FIELD_ACCOUNT: u8 = 5;
const FIELD_CATEGORY: u8 = 6;

/// Generates a deterministic, cryptographically secure fingerprint for an import record.
///
/// This uses Blake3 hashing over the normalized semantic fields of the record.
/// The resulting fingerprint is uniquely tied to the transaction data and is
/// safe to use as an idempotent key during ledger ingestion.
///
/// # Examples
///
/// ```
/// use logos_import::{ImportRecord, deterministic_fingerprint};
///
/// let record1 = ImportRecord::new("src", "2023", 100, "Memo", "A", "C");
/// let record2 = ImportRecord::new("src", "2023", 100, "memo", "A", "C"); // Different casing
///
/// // Normalizes text and guarantees identical hashes
/// assert_eq!(deterministic_fingerprint(&record1), deterministic_fingerprint(&record2));
/// ```
#[must_use]
pub fn deterministic_fingerprint(record: &ImportRecord) -> String {
    let mut hasher = Hasher::new();
    hasher.update(FINGERPRINT_V2_DOMAIN);
    hasher.update(b"\n");
    hash_text_field(&mut hasher, FIELD_SOURCE_ID, record.source_id());
    hash_text_field(&mut hasher, FIELD_TIMESTAMP, record.timestamp());
    hash_amount_field(&mut hasher, FIELD_AMOUNT_CENTS, record.amount_cents());
    hash_text_field(&mut hasher, FIELD_MEMO, record.memo());
    hash_text_field(&mut hasher, FIELD_ACCOUNT, record.account());
    hash_text_field(&mut hasher, FIELD_CATEGORY, record.category());
    hasher.finalize().to_hex().to_string()
}

/// Legacy fingerprint retained only to preserve dedupe compatibility with
/// previously persisted import keys.
#[must_use]
pub fn deterministic_fingerprint_legacy_v1(record: &ImportRecord) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    record
        .source_id()
        .trim()
        .to_ascii_lowercase()
        .hash(&mut hasher);
    record
        .timestamp()
        .trim()
        .to_ascii_lowercase()
        .hash(&mut hasher);
    record.amount_cents().hash(&mut hasher);
    record.memo().trim().to_ascii_lowercase().hash(&mut hasher);
    record
        .account()
        .trim()
        .to_ascii_lowercase()
        .hash(&mut hasher);
    record
        .category()
        .trim()
        .to_ascii_lowercase()
        .hash(&mut hasher);
    hasher.finish()
}

fn hash_text_field(hasher: &mut Hasher, field_tag: u8, value: &str) {
    let normalized = normalize_text(value);
    let len = u64::try_from(normalized.len()).unwrap_or(u64::MAX);
    hasher.update(&[field_tag]);
    hasher.update(&len.to_le_bytes());
    hasher.update(normalized.as_bytes());
}

fn hash_amount_field(hasher: &mut Hasher, field_tag: u8, amount_cents: i64) {
    hasher.update(&[field_tag]);
    hasher.update(&amount_cents.to_le_bytes());
}

fn normalize_text(value: &str) -> String {
    value.trim().to_lowercase()
}
