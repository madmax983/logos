use std::hash::{Hash, Hasher};

use crate::csv::ImportRecord;

#[must_use]
pub fn deterministic_fingerprint(record: &ImportRecord) -> u64 {
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
