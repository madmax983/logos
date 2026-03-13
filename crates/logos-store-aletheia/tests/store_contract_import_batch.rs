use logos_store_aletheia::{AletheiaStore, model::NewImportRecord};

#[test]
fn empty_store_has_zero_import_counts() {
    let store = AletheiaStore::new();
    assert_eq!(store.import_record_count(), 0);
    assert_eq!(store.statement_line_count(), 0);
    assert!(!store.has_import_record_content_hash("non-existent"));
    assert_eq!(store.import_records().count(), 0);
    assert_eq!(store.import_batches().count(), 0);
}

#[test]
fn write_import_batch_yields_expected_records_and_batches() {
    let mut store = AletheiaStore::new();
    store
        .write_import_batch(
            "csv",
            "inline:csv",
            "batch-1",
            0,
            false,
            false,
            &[NewImportRecord::new("hash1", None), NewImportRecord::new("hash2", None)],
        )
        .unwrap();

    assert_eq!(store.import_record_count(), 2);
    let mut records = store.import_records();
    let first = records.next().unwrap();
    let second = records.next().unwrap();
    assert!(first.content_hash_key() == "hash1" || first.content_hash_key() == "hash2");
    assert!(second.content_hash_key() == "hash1" || second.content_hash_key() == "hash2");
    assert_ne!(first.content_hash_key(), second.content_hash_key());
    assert!(records.next().is_none());

    let mut batches = store.import_batches();
    let batch = batches.next().unwrap();
    assert_eq!(batch.batch_key(), "batch-1");
    assert!(batches.next().is_none());
}

#[test]
fn write_import_batch_fails_when_import_kind_is_empty() {
    let mut store = AletheiaStore::new();
    let err = store
        .write_import_batch("", "inline:csv", "batch-1", 0, false, false, &[])
        .unwrap_err();
    assert!(err.to_string().contains("import_kind must not be empty"));
}

#[test]
fn write_import_batch_fails_when_source_uri_is_empty() {
    let mut store = AletheiaStore::new();
    let err = store
        .write_import_batch("csv", "", "batch-1", 0, false, false, &[])
        .unwrap_err();
    assert!(err.to_string().contains("source_uri must not be empty"));
}

#[test]
fn write_import_batch_fails_when_batch_key_is_empty() {
    let mut store = AletheiaStore::new();
    let err = store
        .write_import_batch("csv", "inline:csv", "", 0, false, false, &[])
        .unwrap_err();
    assert!(err.to_string().contains("batch_key must not be empty"));
}

#[test]
fn write_import_batch_fails_when_duplicate_count_is_negative() {
    let mut store = AletheiaStore::new();
    let err = store
        .write_import_batch("csv", "inline:csv", "batch-1", -1, false, false, &[])
        .unwrap_err();
    assert!(
        err.to_string()
            .contains("duplicate_count must be non-negative")
    );
}

#[test]
fn write_import_batch_fails_when_record_hash_key_is_empty() {
    let mut store = AletheiaStore::new();
    let err = store
        .write_import_batch(
            "csv",
            "inline:csv",
            "batch-1",
            0,
            false,
            false,
            &[NewImportRecord::new("", None)],
        )
        .unwrap_err();
    assert!(
        err.to_string()
            .contains("content_hash_key must not be empty")
    );
}

#[test]
fn write_import_batch_fails_when_duplicate_hash_keys_in_payload() {
    let mut store = AletheiaStore::new();
    let err = store
        .write_import_batch(
            "csv",
            "inline:csv",
            "batch-1",
            0,
            false,
            false,
            &[
                NewImportRecord::new("hash1", None),
                NewImportRecord::new("hash1", None),
            ],
        )
        .unwrap_err();
    assert!(
        err.to_string()
            .contains("duplicate import record content_hash_key")
    );
}

#[test]
fn write_import_batch_fails_when_hash_key_already_exists() {
    let mut store = AletheiaStore::new();
    store
        .write_import_batch(
            "csv",
            "inline:csv",
            "batch-1",
            0,
            false,
            false,
            &[NewImportRecord::new("hash1", None)],
        )
        .unwrap();

    let err = store
        .write_import_batch(
            "csv",
            "inline:csv",
            "batch-2",
            0,
            false,
            false,
            &[NewImportRecord::new("hash1", None)],
        )
        .unwrap_err();
    assert!(err.to_string().contains("already exists"));
}

#[test]
fn write_import_batch_fails_when_statement_line_source_uri_is_empty() {
    let mut store = AletheiaStore::new();
    let record = NewImportRecord::with_statement_line("hash1", None, "", "2026-03-01", "memo", 100);
    let err = store
        .write_import_batch("csv", "inline:csv", "batch-1", 0, false, false, &[record])
        .unwrap_err();
    assert!(err.to_string().contains("source_uri must not be empty"));
}

#[test]
fn write_import_batch_fails_when_statement_line_timestamp_is_empty() {
    let mut store = AletheiaStore::new();
    let record = NewImportRecord::with_statement_line("hash1", None, "uri", "", "memo", 100);
    let err = store
        .write_import_batch("csv", "inline:csv", "batch-1", 0, false, false, &[record])
        .unwrap_err();
    assert!(err.to_string().contains("timestamp must not be empty"));
}

#[test]
fn write_import_batch_fails_when_statement_line_memo_is_empty() {
    let mut store = AletheiaStore::new();
    let record = NewImportRecord::with_statement_line("hash1", None, "uri", "2026-03-01", "", 100);
    let err = store
        .write_import_batch("csv", "inline:csv", "batch-1", 0, false, false, &[record])
        .unwrap_err();
    assert!(err.to_string().contains("memo must not be empty"));
}
