use logos_core::{AccountId, Posting, TransactionBuilder};
use logos_store::{
    LedgerStore, MemoryStore, NewImportRecord, StoredFetchArtifactFormat, StoredFetchRunStatus,
};

fn balanced_txn() -> TransactionBuilder {
    TransactionBuilder::new("desc")
        .posting(Posting::debit(AccountId::new("assets:checking").unwrap(), 100).unwrap())
        .posting(Posting::credit(AccountId::new("income:salary").unwrap(), 100).unwrap())
}

#[test]
fn kill_memory_store_getters_and_counts() {
    let mut store = MemoryStore::default();

    let txn = store.write_transaction(balanced_txn()).unwrap();
    assert!(store.has_transaction(&txn));

    store
        .write_budget_target("2026-03", "expenses:food", 500)
        .unwrap();
    assert_eq!(store.budget_targets().len(), 1);

    store
        .write_analytics_artifact_manifest("report", "file", "hash", 1, 2, 0, 0, None)
        .unwrap();
    assert_eq!(store.analytics_artifacts().len(), 1);

    let _batch = store
        .write_import_batch(
            "csv",
            "inline",
            "key",
            0,
            false,
            false,
            &[NewImportRecord::with_statement_line(
                "hash-1",
                Some(&txn),
                "inline",
                "2026-02-01T00:00:00",
                "memo",
                100,
            )],
        )
        .unwrap();

    assert_eq!(store.import_record_count(), 1);
    assert!(store.has_import_record_content_hash("hash-1"));
    assert!(!store.has_import_record_content_hash("missing"));
    assert_eq!(store.import_records().len(), 1);
    assert_eq!(store.import_batches().len(), 1);
    assert_eq!(store.statement_line_count(), 1);

    store
        .write_fetch_run(
            "source-1",
            "bank-1",
            "assets:checking",
            "2026-03",
            StoredFetchRunStatus::Downloaded,
            Some("/tmp/fetch.csv"),
            Some(StoredFetchArtifactFormat::Csv),
            Some(98_766),
            Some(100_000),
            None,
        )
        .unwrap();
    assert_eq!(store.fetch_run_count(), 1);
    assert_eq!(store.fetch_runs().len(), 1);

    store
        .write_reconciliation_run(
            "2026-03",
            "assets:checking",
            0,
            0,
            0,
            0,
            0,
            true,
            0,
            0,
            0,
            std::slice::from_ref(&txn),
        )
        .unwrap();
    assert_eq!(store.reconciliation_run_count(), 1);
    assert!(store.reconciliation_run("recon-1").is_some());
    assert!(store.reconciliation_run("recon-99").is_none());
    assert_eq!(store.reconciliation_runs().len(), 1);

    store
        .write_month_close("2026-03", "assets:checking", "recon-1", None)
        .unwrap();
    assert_eq!(store.month_close_count(), 1);
    assert!(store.month_close("close-1").is_some());
    assert!(store.month_close("close-99").is_none());
    assert_eq!(store.month_closes().len(), 1);
}

#[test]
fn memory_store_mutants_edge_cases() {
    let mut store = MemoryStore::default();

    // Test count replacements with 1 (by adding 2 of each)
    let txn1 = store.write_transaction(balanced_txn()).unwrap();
    let txn2 = store.write_transaction(balanced_txn()).unwrap();

    assert!(store.has_transaction(&txn1));
    assert!(store.has_transaction(&txn2));
    assert!(!store.has_transaction(&logos_core::TransactionId::new("missing").unwrap()));

    store
        .write_import_batch(
            "csv",
            "inline",
            "key1",
            0,
            false,
            false,
            &[NewImportRecord::with_statement_line(
                "hash-1",
                Some(&txn1),
                "inline",
                "2026-02-01T00:00:00",
                "memo",
                100,
            )],
        )
        .unwrap();
    store
        .write_import_batch(
            "csv",
            "inline",
            "key2",
            0,
            false,
            false,
            &[NewImportRecord::with_statement_line(
                "hash-2",
                Some(&txn2),
                "inline",
                "2026-02-01T00:00:00",
                "memo",
                100,
            )],
        )
        .unwrap();

    assert_eq!(store.import_record_count(), 2);
    assert_eq!(store.statement_line_count(), 2);

    store
        .write_fetch_run(
            "src",
            "bank",
            "assets",
            "2026",
            StoredFetchRunStatus::Downloaded,
            Some("file"),
            Some(StoredFetchArtifactFormat::Csv),
            Some(1),
            Some(2),
            None,
        )
        .unwrap();
    store
        .write_fetch_run(
            "src",
            "bank",
            "assets",
            "2026",
            StoredFetchRunStatus::Downloaded,
            Some("file"),
            Some(StoredFetchArtifactFormat::Csv),
            Some(1),
            Some(2),
            None,
        )
        .unwrap();

    assert_eq!(store.fetch_run_count(), 2);

    store
        .write_reconciliation_run(
            "2026-01",
            "assets",
            0,
            0,
            0,
            0,
            0,
            true,
            0,
            0,
            0,
            std::slice::from_ref(&txn1),
        )
        .unwrap();
    store
        .write_reconciliation_run(
            "2026-02",
            "assets",
            0,
            0,
            0,
            0,
            0,
            true,
            0,
            0,
            0,
            std::slice::from_ref(&txn1),
        )
        .unwrap();

    assert_eq!(store.reconciliation_run_count(), 2);

    store
        .write_month_close("2026-01", "assets", "recon-1", None)
        .unwrap();
    store
        .write_month_close("2026-02", "assets", "recon-2", None)
        .unwrap();

    assert_eq!(store.month_close_count(), 2);
}

#[test]
fn memory_store_mutants_edge_cases_pt2() {
    let mut store = MemoryStore::default();

    // Testing validation rules and condition checks
    let txn1 = store.write_transaction(balanced_txn()).unwrap();
    let txn2 = store.write_transaction(balanced_txn()).unwrap();

    let _err = store
        .write_import_batch(
            "csv",
            "inline",
            "key",
            -1,
            false,
            false,
            &[
                NewImportRecord::with_statement_line(
                    "hash-1",
                    Some(&txn1),
                    "inline",
                    "2026-02-01T00:00:00",
                    "memo",
                    100,
                ),
                NewImportRecord::new("hash-2", Some(&txn2)),
            ],
        )
        .unwrap_err();

    // write_fetch_run || checks (should return error when one of the requires is missing and status is downloaded)
    let _err = store
        .write_fetch_run(
            "src",
            "bank",
            "assets",
            "2026",
            StoredFetchRunStatus::Downloaded,
            None,
            Some(StoredFetchArtifactFormat::Csv),
            Some(1),
            Some(2),
            None,
        )
        .unwrap_err();
    let _err2 = store
        .write_fetch_run(
            "src",
            "bank",
            "assets",
            "2026",
            StoredFetchRunStatus::Downloaded,
            Some("file"),
            None,
            Some(1),
            Some(2),
            None,
        )
        .unwrap_err();
    let _err3 = store
        .write_fetch_run(
            "src",
            "bank",
            "assets",
            "2026",
            StoredFetchRunStatus::Downloaded,
            Some("file"),
            Some(StoredFetchArtifactFormat::Csv),
            None,
            Some(2),
            None,
        )
        .unwrap_err();
    let _err5 = store
        .write_fetch_run(
            "src",
            "bank",
            "assets",
            "2026",
            StoredFetchRunStatus::Downloaded,
            Some("file"),
            Some(StoredFetchArtifactFormat::Csv),
            Some(1),
            None,
            None,
        )
        .unwrap_err();

    // write_reconciliation_run lengths check
    let _err = store
        .write_reconciliation_run(
            "2026",
            "assets",
            0,
            0,
            0,
            0,
            0,
            true,
            -1,
            0,
            0,
            &[txn1.clone(), txn2.clone()],
        )
        .unwrap_err();
    let _err2 = store
        .write_reconciliation_run(
            "2026",
            "assets",
            0,
            0,
            0,
            0,
            0,
            true,
            0,
            -1,
            0,
            &[txn1.clone(), txn2.clone()],
        )
        .unwrap_err();
    let _err3 = store
        .write_reconciliation_run(
            "2026",
            "assets",
            0,
            0,
            0,
            0,
            0,
            true,
            0,
            0,
            -1,
            &[txn1.clone(), txn2.clone()],
        )
        .unwrap_err();

    // write_reconciliation_run_and_month_close lengths check
    let _err = store
        .write_reconciliation_run_and_month_close(
            "2026",
            "assets",
            0,
            0,
            0,
            0,
            0,
            true,
            -1,
            0,
            0,
            &[txn1.clone(), txn2.clone()],
            None,
        )
        .unwrap_err();
    let _err2 = store
        .write_reconciliation_run_and_month_close(
            "2026",
            "assets",
            0,
            0,
            0,
            0,
            0,
            true,
            0,
            -1,
            0,
            &[txn1.clone(), txn2.clone()],
            None,
        )
        .unwrap_err();
    let _err3 = store
        .write_reconciliation_run_and_month_close(
            "2026",
            "assets",
            0,
            0,
            0,
            0,
            0,
            true,
            0,
            0,
            -1,
            &[txn1.clone(), txn2.clone()],
            None,
        )
        .unwrap_err();
}

#[test]
fn memory_store_mutants_edge_cases_pt3() {
    let mut store = MemoryStore::default();

    // 474:16: delete ! in write_analytics_artifact_manifest_us (requires artifact_id to be in analytics_artifacts or something?)
    let _ = store
        .write_analytics_artifact_manifest("report", "file", "hash", 1, 2, 0, 0, None)
        .unwrap();
    // Wait, the logic is `if let Some(supersedes_artifact_id) = supersedes_artifact_id && !self.analytics_artifacts.contains_key(supersedes_artifact_id)`
    let err = store
        .write_analytics_artifact_manifest(
            "report",
            "file",
            "hash3",
            1,
            2,
            0,
            0,
            Some("artifact-99"),
        )
        .unwrap_err();
    assert!(matches!(
        err,
        logos_store::StoreError::UnknownArtifact { .. }
    ));
    let _ = store
        .write_analytics_artifact_manifest(
            "report",
            "file",
            "hash4",
            1,
            2,
            0,
            0,
            Some("artifact-1"),
        )
        .unwrap(); // Should pass

    // 669:69: delete ! in write_fetch_run (requires error_summary string empty check?)
    // normalized_error_summary = error_summary.filter(|value| !value.is_empty());
    let run1 = store
        .write_fetch_run(
            "src",
            "bank",
            "assets",
            "2026",
            StoredFetchRunStatus::Failed,
            None,
            None,
            None,
            None,
            Some("error msg"),
        )
        .unwrap();
    assert_eq!(run1.error_summary(), Some("error msg"));
    let run2 = store
        .write_fetch_run(
            "src",
            "bank",
            "assets",
            "2026",
            StoredFetchRunStatus::Failed,
            None,
            None,
            None,
            None,
            Some(""),
        )
        .unwrap();
    assert_eq!(run2.error_summary(), None);

    let txn1 = store.write_transaction(balanced_txn()).unwrap();

    // 804:16: delete ! in write_reconciliation_run_and_month_close
    // if let Some(artifact_id) = analytics_artifact_id && !self.analytics_artifacts.contains_key(artifact_id)
    let err = store
        .write_reconciliation_run_and_month_close(
            "2026-03",
            "assets",
            0,
            0,
            0,
            0,
            0,
            true,
            0,
            0,
            0,
            &[],
            Some("missing-artifact"),
        )
        .unwrap_err();
    assert!(matches!(
        err,
        logos_store::StoreError::UnknownArtifact { .. }
    ));

    // 819:16: delete ! in write_reconciliation_run_and_month_close (transaction_exists)
    let err = store
        .write_reconciliation_run_and_month_close(
            "2026-03",
            "assets",
            0,
            0,
            0,
            0,
            0,
            true,
            0,
            0,
            0,
            &[logos_core::TransactionId::new("missing").unwrap()],
            None,
        )
        .unwrap_err();
    assert!(matches!(
        err,
        logos_store::StoreError::UnknownTransaction { .. }
    ));

    // Write a valid one to make sure it doesn't fail
    let _ = store
        .write_reconciliation_run_and_month_close(
            "2026-04",
            "assets",
            0,
            0,
            0,
            0,
            0,
            true,
            0,
            0,
            0,
            &[txn1.clone()],
            Some("artifact-1"),
        )
        .unwrap();

    // 788:29: replace < with <= in write_reconciliation_run_and_month_close (matched_postings < 0 -> <= 0)
    let err = store
        .write_reconciliation_run_and_month_close(
            "2026-05",
            "assets",
            0,
            0,
            0,
            0,
            0,
            true,
            -1,
            0,
            0,
            &[],
            None,
        )
        .unwrap_err();
    let _ = store
        .write_reconciliation_run_and_month_close(
            "2026-05",
            "assets",
            0,
            0,
            0,
            0,
            0,
            true,
            0,
            0,
            0,
            &[],
            None,
        )
        .unwrap(); // 0 is allowed

    // 793:25 and 798:26 (inflow_cents < 0 -> <= 0, outflow_cents < 0 -> <= 0)
    let err = store
        .write_reconciliation_run_and_month_close(
            "2026-06",
            "assets",
            0,
            0,
            0,
            0,
            0,
            true,
            0,
            -1,
            0,
            &[],
            None,
        )
        .unwrap_err();
    let err2 = store
        .write_reconciliation_run_and_month_close(
            "2026-06",
            "assets",
            0,
            0,
            0,
            0,
            0,
            true,
            0,
            0,
            -1,
            &[],
            None,
        )
        .unwrap_err();
    let _ = store
        .write_reconciliation_run_and_month_close(
            "2026-06",
            "assets",
            0,
            0,
            0,
            0,
            0,
            true,
            0,
            0,
            0,
            &[],
            None,
        )
        .unwrap(); // 0 is allowed
}

#[test]
fn memory_store_mutants_edge_cases_pt4() {
    let store = MemoryStore::new_in_memory();
    let store_def = MemoryStore::default();

    // new_in_memory vs Default::default()
    // It's the same, so MemoryStore::new_in_memory -> Self with Default::default() is equivalent.

    // system_time_us -> Timestamp with Default::default()
    // In our tests `store.write_analytics_artifact_manifest` and other methods capture the created_at timestamp.
    // If it's replaced with Default::default() (which is 0), then multiple creates will have 0.
    let mut store = MemoryStore::new_in_memory();

    let artifact = store
        .write_analytics_artifact_manifest("report", "file", "hash", 1, 2, 0, 0, None)
        .unwrap();
    let _store = MemoryStore::new_in_memory();
    let _store_def = MemoryStore::default();
    assert!(
        artifact.created_at() > 0,
        "Created at should not be default/0"
    );
}

#[test]
fn memory_store_mutants_edge_cases_pt5() {
    // Kill MemoryStore::new_in_memory -> Self with Default::default()
    // It's the exact same implementation, so we can't kill it unless we add a test that specifically
    // verifies something that would break if new_in_memory returned Default::default(). But Default::default()
    // calls new_in_memory() or vice versa. They are identical.
    // This is an unviable mutant because the code is semantically identical.

    // Kill system_time_us -> Timestamp with Default::default()
    // Wait, in pt4 we checked `artifact.created_at() > 0`. Default::default() for i64 is 0.
    // If system_time_us is mutated to return 0, then created_at will be 0.
    // Let's make sure it's actually using system_time_us.
    // Let's just sleep for 1ms and make sure the timestamp changes.
    let mut store = MemoryStore::new_in_memory();
    let t1 = store
        .write_analytics_artifact_manifest("report", "file", "hash1", 1, 2, 0, 0, None)
        .unwrap();
    std::thread::sleep(std::time::Duration::from_millis(2));
    let t2 = store
        .write_analytics_artifact_manifest("report", "file", "hash2", 1, 2, 0, 0, None)
        .unwrap();
    assert!(
        t2.created_at() > t1.created_at(),
        "Timestamps should increase, not be 0 always"
    );
}

#[test]
fn memory_store_mutants_edge_cases_pt6() {
    let mut store = MemoryStore::new_in_memory();
    // In MemoryStore next_timestamp_us uses next_id to increment a counter OR system_time_us?
    // Wait, next_timestamp_us logic:
    // let now = system_time_us(); if now > self.last_timestamp { self.last_timestamp = now; now } else { self.last_timestamp += 1; self.last_timestamp }
    // If system_time_us returns 0, the first call returns 0, second returns 1, third returns 2...
    // But since it's monotonic anyway, comparing > 0 doesn't catch it if it just uses the fallback.
    // However, a valid timestamp is going to be >> 1000.
    let t1 = store
        .write_analytics_artifact_manifest("report", "file", "hash1", 1, 2, 0, 0, None)
        .unwrap();
    assert!(
        t1.created_at() > 1_000_000,
        "Should use real system time, not fallback to 0 then increment"
    );
}
