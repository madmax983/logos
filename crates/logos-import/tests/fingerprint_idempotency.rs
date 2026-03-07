use logos_import::{ImportRecord, deterministic_fingerprint, deterministic_fingerprint_legacy_v1};

fn sample_record() -> ImportRecord {
    ImportRecord::new(
        "broker.csv",
        "2026-02-01T09:30:00",
        12_345,
        "RSU sale",
        "assets:checking",
        "income:rsu",
    )
}

#[test]
fn same_record_produces_same_fingerprint() {
    let record = sample_record();

    let a = deterministic_fingerprint(&record);
    let b = deterministic_fingerprint(&record);

    assert_eq!(a, b);
}

#[test]
fn changed_record_produces_different_fingerprint() {
    let base = sample_record();
    let changed = ImportRecord::new(
        "broker.csv",
        "2026-02-01T09:30:00",
        12_346,
        "RSU sale",
        "assets:checking",
        "income:rsu",
    );

    assert_ne!(
        deterministic_fingerprint(&base),
        deterministic_fingerprint(&changed)
    );
}

#[test]
fn unicode_case_variants_produce_same_fingerprint() {
    let upper = ImportRecord::new(
        "broker.csv",
        "2026-02-01T09:30:00",
        12_345,
        "CAF\u{00C9} sale",
        "assets:checking",
        "income:rsu",
    );
    let lower = ImportRecord::new(
        "broker.csv",
        "2026-02-01T09:30:00",
        12_345,
        "caf\u{00E9} sale",
        "assets:checking",
        "income:rsu",
    );

    assert_eq!(
        deterministic_fingerprint(&upper),
        deterministic_fingerprint(&lower)
    );
}

#[test]
fn fingerprint_is_lower_hex_blake3_digest() {
    let value = deterministic_fingerprint(&sample_record());
    assert_eq!(value.len(), 64);
    assert!(value.bytes().all(|byte| byte.is_ascii_hexdigit()));
    assert_eq!(value, value.to_ascii_lowercase());
}

#[test]
fn fingerprint_matches_expected_vector() {
    let value = deterministic_fingerprint(&sample_record());
    assert_eq!(
        value, "3b9daf526b5ef151906c316ee4556e969d6fcf9107da474afa6f8be9d765180f",
        "update this constant only with an intentional fingerprint version migration"
    );
}

#[test]
fn legacy_fingerprint_matches_expected_vector() {
    assert_eq!(
        deterministic_fingerprint_legacy_v1(&sample_record()),
        9_159_060_840_922_382_101
    );
}
