use logos_core::{AccountId, Correction, Posting, TransactionBuilder, TransactionId};
use logos_store::model::{NewImportRecord, StoredFetchArtifactFormat, StoredFetchRunStatus};
use logos_store::{LedgerStore, MemoryStore};

fn balanced_txn(
    description: &str,
    debit_account: &str,
    credit_account: &str,
    amount: i64,
) -> TransactionBuilder {
    TransactionBuilder::new(description)
        .posting(
            Posting::debit(
                AccountId::new(debit_account).expect("debit account"),
                amount,
            )
            .expect("debit"),
        )
        .posting(
            Posting::credit(
                AccountId::new(credit_account).expect("credit account"),
                amount,
            )
            .expect("credit"),
        )
}

#[test]
fn memory_store_allocates_deterministic_ids_and_tracks_related_records() {
    let mut store = MemoryStore::new_in_memory();

    let txn_1 = store
        .write_transaction(balanced_txn(
            "paycheck",
            "assets:checking",
            "income:salary",
            10_000,
        ))
        .expect("write txn 1");
    let txn_2 = store
        .write_transaction(balanced_txn(
            "groceries",
            "expenses:food",
            "assets:checking",
            2_500,
        ))
        .expect("write txn 2");

    let artifact = store
        .write_analytics_artifact_manifest(
            "report",
            "file:///tmp/report.parquet",
            "sha256:artifact",
            1,
            2,
            1_000,
            2_000,
            None,
        )
        .expect("artifact");
    assert_eq!(artifact.artifact_id(), "artifact-1");
    assert_eq!(store.analytics_artifact("artifact-1").unwrap(), artifact);

    store
        .write_budget_target("2026-03", "expenses:food", 500)
        .expect("budget target");
    assert_eq!(
        store
            .budget_target("2026-03", "expenses:food")
            .expect("stored target")
            .budget_cents(),
        500
    );

    let import_batch = store
        .write_import_batch(
            "csv-statement",
            "inline:statement",
            "batch-key-1",
            0,
            false,
            false,
            &[NewImportRecord::with_statement_line(
                "sha256:import-1",
                Some(&txn_1),
                "inline:statement",
                "2026-02-01T00:00:00",
                "coffee shop",
                -1_234,
            )],
        )
        .expect("import batch");
    assert_eq!(import_batch.batch_id(), "import-batch-1");
    assert_eq!(store.import_record_count(), 1);
    assert!(store.has_import_record_content_hash("sha256:import-1"));
    assert_eq!(store.statement_line_count(), 1);
    assert_eq!(store.statement_lines()[0].line_id(), "stmt-line-1");

    let fetch_run = store
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
        .expect("fetch run");
    assert_eq!(fetch_run.run_id(), "fetch-1");
    assert_eq!(store.fetch_run("fetch-1").unwrap(), fetch_run);

    let recon_run = store
        .write_reconciliation_run(
            "2026-03",
            "assets:checking",
            100_000,
            -1_234,
            98_766,
            98_766,
            0,
            true,
            1,
            1_234,
            0,
            std::slice::from_ref(&txn_1),
        )
        .expect("reconciliation run");
    assert_eq!(recon_run.run_id(), "recon-1");
    let linked_lines = store.statement_lines_for_reconciliation_run("recon-1");
    assert_eq!(linked_lines.len(), 1);
    assert_eq!(linked_lines[0].line_id(), "stmt-line-1");

    let close = store
        .write_month_close("2026-03", "assets:checking", "recon-1", Some("artifact-1"))
        .expect("month close");
    assert_eq!(close.close_id(), "close-1");
    assert_eq!(
        store
            .month_close_for_scope("2026-03", "assets:checking")
            .expect("close by scope"),
        close
    );

    assert_eq!(store.transaction_count(), 2);
    assert_eq!(store.correction_count(), 0);
    assert_eq!(store.fetch_run_count(), 1);
    assert_eq!(store.reconciliation_run_count(), 1);
    assert_eq!(store.month_close_count(), 1);
    assert_eq!(store.transactions().len(), 2);
    assert_eq!(store.transactions()[0].id(), &txn_1);
    assert_eq!(store.transactions()[1].id(), &txn_2);
}

#[test]
fn memory_store_current_projection_excludes_superseded_transactions() {
    let mut store = MemoryStore::new();

    let txn_1 = store
        .write_transaction_with_valid_time(
            balanced_txn("paycheck", "assets:checking", "income:salary", 10_000),
            Some(100),
        )
        .expect("txn 1");
    let txn_2 = store
        .write_transaction_with_valid_time(
            balanced_txn("groceries", "expenses:food", "assets:checking", 2_500),
            Some(200),
        )
        .expect("txn 2");

    store
        .write_correction(Correction::new(txn_1.clone(), "fix account").expect("correction"))
        .expect("write correction");

    assert_eq!(store.transaction_count(), 2);
    assert_eq!(store.correction_count(), 1);
    assert_eq!(store.latest_correction().unwrap().supersedes_id(), &txn_1);
    let current = store.transactions();
    assert_eq!(current.len(), 1);
    assert_eq!(current[0].id(), &txn_2);
}

#[test]
fn memory_store_as_of_respects_valid_time() {
    let mut store = MemoryStore::new();

    let txn_1 = store
        .write_transaction_with_valid_time(
            balanced_txn("paycheck", "assets:checking", "income:salary", 10_000),
            Some(100),
        )
        .expect("txn 1");
    let txn_2 = store
        .write_transaction_with_valid_time(
            balanced_txn("groceries", "expenses:food", "assets:checking", 2_500),
            Some(200),
        )
        .expect("txn 2");

    let visible_before = store
        .transactions_as_of_us(150, i64::MAX)
        .expect("as-of before second valid time");
    assert_eq!(visible_before.len(), 1);
    assert_eq!(visible_before[0].id(), &txn_1);

    let visible_after = store
        .transactions_as_of_us(250, i64::MAX)
        .expect("as-of after second valid time");
    assert_eq!(visible_after.len(), 2);
    assert_eq!(visible_after[0].id(), &txn_1);
    assert_eq!(visible_after[1].id(), &txn_2);
}

#[test]
fn memory_store_rejects_unknown_related_records() {
    let mut store = MemoryStore::new_in_memory();
    let missing_txn = TransactionId::new("txn-99").expect("id");

    let correction_err = store
        .write_correction(Correction::new(missing_txn.clone(), "missing").expect("correction"))
        .expect_err("missing correction target");
    assert!(matches!(
        correction_err,
        logos_store::StoreError::UnknownTransaction { .. }
    ));

    let import_err = store
        .write_import_batch(
            "csv-statement",
            "inline:statement",
            "batch-key-1",
            0,
            false,
            false,
            &[NewImportRecord::new("sha256:import-1", Some(&missing_txn))],
        )
        .expect_err("missing imported txn");
    assert!(matches!(
        import_err,
        logos_store::StoreError::UnknownTransaction { .. }
    ));

    let recon_err = store
        .write_reconciliation_run(
            "2026-03",
            "assets:checking",
            100_000,
            0,
            100_000,
            100_000,
            0,
            true,
            0,
            0,
            0,
            std::slice::from_ref(&missing_txn),
        )
        .expect_err("missing reconciled txn");
    assert!(matches!(
        recon_err,
        logos_store::StoreError::UnknownTransaction { .. }
    ));
}
