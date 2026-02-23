use logos_import::{ImportRecord, deterministic_fingerprint};

#[test]
fn same_record_produces_same_fingerprint() {
    let record = ImportRecord::new(
        "broker.csv",
        "2026-02-01T09:30:00",
        12_345,
        "RSU sale",
        "assets:checking",
        "income:rsu",
    );

    let a = deterministic_fingerprint(&record);
    let b = deterministic_fingerprint(&record);

    assert_eq!(a, b);
}

#[test]
fn changed_record_produces_different_fingerprint() {
    let base = ImportRecord::new(
        "broker.csv",
        "2026-02-01T09:30:00",
        12_345,
        "RSU sale",
        "assets:checking",
        "income:rsu",
    );
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
