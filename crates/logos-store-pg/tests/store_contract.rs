#![allow(clippy::too_many_lines)]
use std::time::{SystemTime, UNIX_EPOCH};

use logos_core::{AccountId, Posting, TransactionBuilder, TransactionId};
use logos_store::error::StoreError;
use logos_store::model::{NewImportRecord, StoredFetchArtifactFormat, StoredFetchRunStatus};
use logos_store::traits::LedgerStore;
use logos_store_pg::PostgresStore;
use testcontainers_modules::{postgres, testcontainers::runners::SyncRunner};

fn connect_store(database_url: &str) -> PostgresStore {
    let mut store = PostgresStore::connect(database_url).expect("connect postgres store");
    store.run_migrations().expect("run migrations");
    store
}

fn now_us() -> i64 {
    i64::try_from(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_micros(),
    )
    .expect("microseconds must fit into i64")
}

fn balanced_transaction(description: &str, amount_cents: i64) -> TransactionBuilder {
    TransactionBuilder::new(description)
        .posting(Posting::debit(AccountId::new("assets:checking").unwrap(), amount_cents).unwrap())
        .posting(Posting::credit(AccountId::new("income:salary").unwrap(), amount_cents).unwrap())
}

#[test]
#[ignore = "requires Docker (testcontainers)"]
fn budget_targets_and_artifacts_round_trip() {
    let container = postgres::Postgres::default()
        .start()
        .expect("start postgres container");
    let host = container.get_host().expect("container host");
    let port = container.get_host_port_ipv4(5432).expect("postgres port");
    let database_url = format!("postgres://postgres:postgres@{host}:{port}/postgres");

    let mut store = connect_store(&database_url);

    assert!(store.budget_target("2026-03", "expenses:food").is_none());
    store
        .write_budget_target("2026-03", "expenses:food", 500)
        .expect("write budget target");
    store
        .write_budget_target("2026-03", "expenses:food", 700)
        .expect("overwrite budget target");

    let target = store
        .budget_target("2026-03", "expenses:food")
        .expect("budget target");
    assert_eq!(target.budget_cents(), 700);
    assert_eq!(store.budget_targets().len(), 1);

    let artifact1 = store
        .write_analytics_artifact_manifest(
            "report",
            "s3://bucket/report-1.json",
            "hash-1",
            1,
            10,
            now_us(),
            now_us(),
            None,
        )
        .expect("write first artifact");
    let artifact2 = store
        .write_analytics_artifact_manifest_us(
            "report",
            "s3://bucket/report-2.json",
            "hash-2",
            1,
            12,
            now_us(),
            now_us(),
            Some(artifact1.artifact_id()),
        )
        .expect("write second artifact");

    assert_eq!(store.analytics_artifacts().len(), 2);
    assert_eq!(
        store
            .analytics_artifact(artifact1.artifact_id())
            .expect("first artifact")
            .artifact_uri(),
        "s3://bucket/report-1.json"
    );
    assert_eq!(
        store
            .analytics_artifact(artifact2.artifact_id())
            .expect("second artifact")
            .supersedes_artifact_id(),
        Some(artifact1.artifact_id())
    );

    let err = store
        .write_analytics_artifact_manifest_us(
            "report",
            "s3://bucket/report-3.json",
            "hash-3",
            1,
            13,
            now_us(),
            now_us(),
            Some("missing-artifact"),
        )
        .expect_err("unknown supersedes artifact must fail");
    assert!(matches!(err, StoreError::UnknownArtifact { .. }));
}

#[test]
#[ignore = "requires Docker (testcontainers)"]
fn import_batches_fetch_runs_and_statement_lines_round_trip() {
    let container = postgres::Postgres::default()
        .start()
        .expect("start postgres container");
    let host = container.get_host().expect("container host");
    let port = container.get_host_port_ipv4(5432).expect("postgres port");
    let database_url = format!("postgres://postgres:postgres@{host}:{port}/postgres");

    let mut store = connect_store(&database_url);
    let txn1 = store
        .write_transaction(balanced_transaction("txn-1", 1_000))
        .expect("write txn 1");
    let txn2 = store
        .write_transaction(balanced_transaction("txn-2", 2_000))
        .expect("write txn 2");

    let batch = store
        .write_import_batch(
            "bank_csv",
            "s3://bucket/import.csv",
            "batch-1",
            0,
            false,
            false,
            &[
                NewImportRecord::new("hash-1", Some(&txn1)),
                NewImportRecord::with_statement_line(
                    "hash-2",
                    Some(&txn2),
                    "s3://bucket/import.csv",
                    "2026-03-01T00:00:00",
                    "coffee",
                    -1_250,
                ),
            ],
        )
        .expect("write import batch");

    assert_eq!(store.import_record_count(), 2);
    assert!(store.has_import_record_content_hash("hash-1"));
    assert_eq!(store.import_records().len(), 2);
    assert_eq!(store.import_batches().len(), 1);
    assert_eq!(store.statement_line_count(), 1);
    assert_eq!(store.statement_lines().len(), 1);
    assert_eq!(store.statement_lines()[0].batch_id(), batch.batch_id());
    assert_eq!(
        store.statement_lines()[0]
            .imported_txn_id()
            .map(TransactionId::as_str),
        Some(txn2.as_str())
    );

    let downloaded = store
        .write_fetch_run(
            "plaid",
            "bank-1",
            "assets:checking",
            "2026-03",
            StoredFetchRunStatus::Downloaded,
            Some("/tmp/fetch.csv"),
            Some(StoredFetchArtifactFormat::Csv),
            Some(10_000),
            Some(12_000),
            None,
        )
        .expect("write downloaded fetch run");
    let failed = store
        .write_fetch_run(
            "plaid",
            "bank-1",
            "assets:checking",
            "2026-03",
            StoredFetchRunStatus::Failed,
            None,
            None,
            None,
            None,
            Some("upstream outage"),
        )
        .expect("write failed fetch run");

    assert_eq!(store.fetch_run_count(), 2);
    assert_eq!(
        store
            .fetch_run(downloaded.run_id())
            .expect("downloaded run")
            .status(),
        StoredFetchRunStatus::Downloaded
    );
    assert_eq!(
        store
            .fetch_run(failed.run_id())
            .expect("failed run")
            .error_summary(),
        Some("upstream outage")
    );
    assert_eq!(store.fetch_runs().len(), 2);
}

#[test]
#[ignore = "requires Docker (testcontainers)"]
fn reconciliation_run_and_month_close_round_trip() {
    let container = postgres::Postgres::default()
        .start()
        .expect("start postgres container");
    let host = container.get_host().expect("container host");
    let port = container.get_host_port_ipv4(5432).expect("postgres port");
    let database_url = format!("postgres://postgres:postgres@{host}:{port}/postgres");

    let mut store = connect_store(&database_url);
    let txn1 = store
        .write_transaction(balanced_transaction("txn-1", 1_000))
        .expect("write txn 1");
    let txn2 = store
        .write_transaction(balanced_transaction("txn-2", 2_000))
        .expect("write txn 2");

    store
        .write_import_batch(
            "bank_csv",
            "s3://bucket/import.csv",
            "batch-1",
            0,
            false,
            false,
            &[
                NewImportRecord::with_statement_line(
                    "hash-1",
                    Some(&txn1),
                    "s3://bucket/import.csv",
                    "2026-03-01T00:00:00",
                    "coffee",
                    -1_250,
                ),
                NewImportRecord::with_statement_line(
                    "hash-2",
                    Some(&txn2),
                    "s3://bucket/import.csv",
                    "2026-03-02T00:00:00",
                    "groceries",
                    -2_500,
                ),
            ],
        )
        .expect("write import batch");

    let run = store
        .write_reconciliation_run(
            "2026-03",
            "assets:checking",
            10_000,
            -3_750,
            6_250,
            6_250,
            0,
            true,
            2,
            0,
            3_750,
            &[txn1.clone(), txn2.clone()],
        )
        .expect("write reconciliation run");

    assert_eq!(store.reconciliation_run_count(), 1);
    assert_eq!(store.reconciliation_runs().len(), 1);
    assert_eq!(
        store
            .reconciliation_run(run.run_id())
            .expect("reconciliation run")
            .matched_transaction_count(),
        2
    );
    let statement_lines = store.statement_lines_for_reconciliation_run(run.run_id());
    assert_eq!(statement_lines.len(), 2);
    assert!(
        statement_lines
            .windows(2)
            .all(|pair| pair[0].line_id() <= pair[1].line_id())
    );

    let artifact = store
        .write_analytics_artifact_manifest_us(
            "month-close",
            "s3://bucket/month-close.json",
            "hash-close",
            1,
            2,
            now_us(),
            now_us(),
            None,
        )
        .expect("write analytics artifact");

    let combined = store
        .write_reconciliation_run_and_month_close(
            "2026-04",
            "assets:savings",
            8_000,
            -2_000,
            6_000,
            6_000,
            0,
            true,
            1,
            0,
            2_000,
            std::slice::from_ref(&txn1),
            Some(artifact.artifact_id()),
        )
        .expect("write reconciliation run and month close");

    assert_eq!(store.month_close_count(), 1);
    assert_eq!(
        store
            .month_close(combined.1.close_id())
            .expect("combined close")
            .reconciliation_run_id(),
        combined.0.run_id()
    );
    assert_eq!(
        store
            .month_close_for_scope("2026-04", "assets:savings")
            .expect("close by scope")
            .close_id(),
        combined.1.close_id()
    );

    let standalone_run = store
        .write_reconciliation_run(
            "2026-05",
            "assets:brokerage",
            5_000,
            -1_000,
            4_000,
            4_000,
            0,
            true,
            1,
            0,
            1_000,
            std::slice::from_ref(&txn2),
        )
        .expect("write standalone reconciliation run");
    let standalone_close = store
        .write_month_close(
            "2026-05",
            "assets:brokerage",
            standalone_run.run_id(),
            Some(artifact.artifact_id()),
        )
        .expect("write standalone month close");

    assert_eq!(store.month_close_count(), 2);
    assert_eq!(store.month_closes().len(), 2);
    assert_eq!(
        store
            .month_close(standalone_close.close_id())
            .expect("standalone close")
            .reconciliation_run_id(),
        standalone_run.run_id()
    );
    assert_eq!(
        store
            .month_close_for_scope("2026-05", "assets:brokerage")
            .expect("standalone close by scope")
            .close_id(),
        standalone_close.close_id()
    );
}
