use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::DateTime;
use logos_cli::runtime::{CliRuntime, MonthAutopilotRequest};
use logos_core::TransactionId;
use logos_import::CsvMapping;

fn temp_runtime_path(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!("logos-cli-runtime-{prefix}-{nanos}.db"))
}

fn cleanup_runtime_path(path: &Path) {
    if path.exists() {
        let _ = std::fs::remove_dir_all(path);
    }
}

fn cleanup_file(path: &Path) {
    if path.exists() {
        let _ = std::fs::remove_file(path);
    }
}

fn timestamp_micros(iso8601: &str) -> i64 {
    DateTime::parse_from_rfc3339(iso8601)
        .expect("parse timestamp")
        .timestamp_micros()
}

#[test]
fn e2e_happy_path_posts_and_reports_register_balance() {
    let mut runtime = CliRuntime::new_in_memory();

    let txn_id = runtime
        .post_double_entry("paycheck", "assets:checking", "income:salary", 10_000)
        .expect("post");

    assert!(runtime.transaction_exists(&txn_id));
    assert_eq!(runtime.register_balance_for("assets:checking"), 10_000);
}

#[test]
fn e2e_import_is_idempotent_by_fingerprint() {
    let mut runtime = CliRuntime::new_in_memory();
    let mapping = CsvMapping::default();
    let line = "2026-02-01T09:30:00,12345,RSU sale,assets:checking,income:rsu";

    let inserted_first = runtime
        .import_csv_row(line, &mapping)
        .expect("first import");
    let inserted_second = runtime
        .import_csv_row(line, &mapping)
        .expect("second import");

    assert!(inserted_first);
    assert!(!inserted_second);
    assert_eq!(runtime.imported_record_count(), 1);
}

#[test]
fn e2e_import_csv_idempotency_persists_across_reopen() {
    let path = temp_runtime_path("csv-reopen-idempotency");
    let mapping = CsvMapping::default();
    let line = "2026-02-01T09:30:00,12345,RSU sale,assets:checking,income:rsu";

    {
        let mut runtime = CliRuntime::open(&path).expect("open");
        let inserted = runtime
            .import_csv_row(line, &mapping)
            .expect("first import");
        assert!(inserted);
    }

    {
        let mut reopened = CliRuntime::open(&path).expect("reopen");
        let inserted = reopened
            .import_csv_row(line, &mapping)
            .expect("second import");
        assert!(!inserted);
    }

    cleanup_runtime_path(&path);
}

#[test]
fn e2e_runtime_reopen_restores_persisted_transactions() {
    let path = temp_runtime_path("reopen");
    {
        let mut runtime = CliRuntime::open(&path).expect("open");
        let txn_id = runtime
            .post_double_entry("paycheck", "assets:checking", "income:salary", 10_000)
            .expect("post");
        assert!(runtime.transaction_exists(&txn_id));
    }

    let reopened = CliRuntime::open(&path).expect("reopen");
    assert!(reopened.transaction_exists(&TransactionId::new("txn-1")));
    assert_eq!(reopened.register_balance_for("assets:checking"), 10_000);

    cleanup_runtime_path(&path);
}

#[test]
fn e2e_runtime_month_report_and_budget_variance_use_posted_transactions() {
    let mut runtime = CliRuntime::new_in_memory();
    let month_key = CliRuntime::current_month_key_utc();

    runtime
        .post_double_entry("paycheck", "assets:checking", "income:salary", 10_000)
        .expect("post income");
    runtime
        .post_double_entry("groceries", "expenses:food", "assets:checking", 2_500)
        .expect("post expense");

    let report = runtime.month_report_for("assets:checking", &month_key);
    assert_eq!(report.checking_balance_cents(), 7_500);
    assert_eq!(report.income_cents(), 10_000);
    assert_eq!(report.expense_cents(), 2_500);
    assert_eq!(report.cashflow_cents(), 7_500);
    assert_eq!(
        runtime.budget_variance_for_month(&month_key, 3_000, "expenses:"),
        500
    );

    let empty_month_report = runtime.month_report_for("assets:checking", "1900-01");
    assert_eq!(empty_month_report.checking_balance_cents(), 0);
    assert_eq!(empty_month_report.income_cents(), 0);
    assert_eq!(empty_month_report.expense_cents(), 0);
    assert_eq!(empty_month_report.cashflow_cents(), 0);
}

#[test]
fn e2e_runtime_rsu_budget_plan_returns_conservative_baseline() {
    let runtime = CliRuntime::new_in_memory();
    let month_key = CliRuntime::current_month_key_utc();

    let plan = runtime
        .plan_rsu_budget_for_month(&month_key, 300, 45, 10_000, 12_000, 16_000, 250_000, 60, 30)
        .expect("plan");
    let bear = plan.bear().expect("bear");

    assert_eq!(plan.month_key(), month_key);
    assert_eq!(
        plan.conservative_budget_cents(),
        bear.monthly_income_cents()
    );
    assert!(plan.base().expect("base").reserve_sweep_cents() >= 0);
}

#[test]
fn e2e_runtime_reconcile_month_computes_match_and_variance() {
    let mut runtime = CliRuntime::new_in_memory();
    let month_key = CliRuntime::current_month_key_utc();

    runtime
        .post_double_entry("paycheck", "assets:checking", "income:salary", 10_000)
        .expect("post income");
    runtime
        .post_double_entry("groceries", "expenses:food", "assets:checking", 2_500)
        .expect("post expense");

    let matched = runtime.reconcile_month_for("assets:checking", &month_key, 100_000, 107_500);
    assert_eq!(matched.ledger_delta_cents(), 7_500);
    assert_eq!(matched.expected_closing_balance_cents(), 107_500);
    assert_eq!(matched.variance_cents(), 0);
    assert!(matched.is_reconciled());

    let mismatched = runtime.reconcile_month_for("assets:checking", &month_key, 100_000, 106_000);
    assert_eq!(mismatched.expected_closing_balance_cents(), 107_500);
    assert_eq!(mismatched.variance_cents(), -1_500);
    assert!(!mismatched.is_reconciled());
}

#[test]
fn e2e_runtime_reconciliation_run_persists_and_run_ids_continue_after_reopen() {
    let path = temp_runtime_path("reconcile-reopen");
    let month_key = CliRuntime::current_month_key_utc();

    {
        let mut runtime = CliRuntime::open(&path).expect("open");
        runtime
            .post_double_entry("paycheck", "assets:checking", "income:salary", 10_000)
            .expect("post income");
        runtime
            .post_double_entry("groceries", "expenses:food", "assets:checking", 2_500)
            .expect("post expense");

        let first = runtime
            .reconcile_and_persist_month_for("assets:checking", &month_key, 100_000, 106_000)
            .expect("persist first reconciliation");
        assert_eq!(first.run_id(), "recon-1");
        assert_eq!(first.ledger_delta_cents(), 7_500);
        assert_eq!(first.variance_cents(), -1_500);
        assert_eq!(first.matched_postings(), 2);
        assert_eq!(first.matched_transaction_count(), 2);
        assert_eq!(runtime.reconciliation_run_count(), 1);
        assert_eq!(
            runtime
                .reconciliation_run("recon-1")
                .expect("first run exists")
                .run_id(),
            "recon-1"
        );
    }

    {
        let mut reopened = CliRuntime::open(&path).expect("reopen");
        assert_eq!(reopened.reconciliation_run_count(), 1);
        assert_eq!(
            reopened
                .reconciliation_run("recon-1")
                .expect("reloaded first run")
                .matched_transaction_count(),
            2
        );

        let second = reopened
            .reconcile_and_persist_month_for("assets:checking", &month_key, 100_000, 107_500)
            .expect("persist second reconciliation");
        assert_eq!(second.run_id(), "recon-2");
        assert!(second.reconciled());
        assert_eq!(reopened.reconciliation_run_count(), 2);
    }

    cleanup_runtime_path(&path);
}

#[test]
fn e2e_runtime_reconcile_list_filters_and_sorts_latest_first() {
    let path = temp_runtime_path("reconcile-list");
    {
        let mut runtime = CliRuntime::open(&path).expect("open");
        let month_key = CliRuntime::current_month_key_utc();
        runtime
            .post_double_entry("paycheck", "assets:checking", "income:salary", 10_000)
            .expect("post");
        runtime
            .reconcile_and_persist_month_for("assets:checking", &month_key, 100_000, 110_000)
            .expect("first run");
        runtime
            .reconcile_and_persist_month_for("assets:checking", &month_key, 110_000, 110_000)
            .expect("second run");

        let all_runs = runtime.list_reconciliation_runs(None, None);
        assert_eq!(all_runs.len(), 2);
        assert_eq!(all_runs[0].run_id(), "recon-2");
        assert_eq!(all_runs[1].run_id(), "recon-1");

        let filtered = runtime.list_reconciliation_runs(Some(&month_key), Some("assets:checking"));
        assert_eq!(filtered.len(), 2);

        let no_match = runtime.list_reconciliation_runs(Some("1900-01"), None);
        assert!(no_match.is_empty());
    }

    cleanup_runtime_path(&path);
}

#[test]
fn e2e_runtime_month_close_persists_and_blocks_duplicate_scope_close() {
    let path = temp_runtime_path("month-close");
    let month_key = CliRuntime::current_month_key_utc();
    let run_id;

    {
        let mut runtime = CliRuntime::open(&path).expect("open");
        runtime
            .post_double_entry("paycheck", "assets:checking", "income:salary", 10_000)
            .expect("post");
        run_id = runtime
            .reconcile_and_persist_month_for("assets:checking", &month_key, 100_000, 110_000)
            .expect("run")
            .run_id()
            .to_owned();

        let close = runtime
            .close_month(&month_key, "assets:checking", &run_id, None)
            .expect("close");
        assert_eq!(close.close_id(), "close-1");

        let duplicate_err = runtime
            .close_month(&month_key, "assets:checking", &run_id, None)
            .expect_err("duplicate close");
        assert!(duplicate_err.to_string().contains("already closed"));
    }

    {
        let reopened = CliRuntime::open(&path).expect("reopen");
        let close = reopened
            .month_close_for_scope(&month_key, "assets:checking")
            .expect("reloaded close");
        assert_eq!(close.reconciliation_run_id(), run_id);
        assert_eq!(close.month_key(), month_key);
    }

    cleanup_runtime_path(&path);
}

#[test]
fn e2e_runtime_month_autopilot_runs_import_reconcile_report_and_close() {
    let path = temp_runtime_path("month-autopilot");
    let statement_path = temp_runtime_path("month-autopilot-statement")
        .with_extension("pdf")
        .to_string_lossy()
        .to_string();
    std::fs::write(
        &statement_path,
        "2026-02-01 COFFEE SHOP -12.34\n2026-02-02 PAYROLL 1000.00\n",
    )
    .expect("write statement");

    {
        let mut runtime = CliRuntime::open(&path).expect("open");
        let request = MonthAutopilotRequest::new("2026-02", "assets:checking", 100_000, 198_766)
            .with_statement_pdf(&statement_path)
            .with_confirm_close(true);
        let summary = runtime
            .run_month_autopilot(&request)
            .expect("autopilot succeeds");

        assert_eq!(summary.month_key(), "2026-02");
        assert_eq!(summary.checking_account(), "assets:checking");
        assert_eq!(summary.imported_count(), 2);
        assert_eq!(summary.duplicate_count(), 0);
        assert_eq!(summary.reconciliation_run().variance_cents(), 0);
        assert!(summary.reconciliation_run().reconciled());
        assert_eq!(summary.report().checking_balance_cents(), 98_766);
        assert_eq!(summary.report().income_cents(), 100_000);
        assert_eq!(summary.report().expense_cents(), 1_234);
        assert_eq!(summary.report().cashflow_cents(), 98_766);
    }

    {
        let reopened = CliRuntime::open(&path).expect("reopen");
        assert_eq!(reopened.reconciliation_run_count(), 1);
        assert!(
            reopened
                .month_close_for_scope("2026-02", "assets:checking")
                .is_some()
        );
    }

    cleanup_file(Path::new(&statement_path));
    cleanup_runtime_path(&path);
}

#[test]
fn e2e_runtime_month_autopilot_requires_confirm_close() {
    let mut runtime = CliRuntime::new_in_memory();
    let request = MonthAutopilotRequest::new("2026-02", "assets:checking", 100_000, 100_000);

    let err = runtime
        .run_month_autopilot(&request)
        .expect_err("confirm-close is required");
    assert!(
        err.to_string()
            .contains("requires --confirm-close to persist month close")
    );
    assert_eq!(runtime.reconciliation_run_count(), 0);
    assert!(
        runtime
            .month_close_for_scope("2026-02", "assets:checking")
            .is_none()
    );
}

#[test]
fn e2e_runtime_reopen_restores_persisted_budget_targets() {
    let path = temp_runtime_path("reopen-budget-target");
    {
        let mut runtime = CliRuntime::open(&path).expect("open");
        runtime
            .set_budget_target_for_month("2026-03", "expenses:food", 250_000)
            .expect("set budget");
    }

    let reopened = CliRuntime::open(&path).expect("reopen");
    assert_eq!(
        reopened.budget_target_for_month("2026-03", "expenses:food"),
        Some(250_000)
    );

    cleanup_runtime_path(&path);
}

#[test]
fn e2e_pdf_import_posts_transactions_and_deduplicates() {
    let mut runtime = CliRuntime::new_in_memory();
    let statement_path = temp_runtime_path("pdf-import")
        .with_extension("pdf")
        .to_string_lossy()
        .to_string();
    std::fs::write(
        &statement_path,
        "2026-02-01 COFFEE SHOP -12.34\n2026-02-02 PAYROLL 1000.00\n",
    )
    .expect("write statement");

    let summary = runtime
        .import_pdf_statement(&statement_path, "assets:checking", false, false)
        .expect("import");
    assert_eq!(summary.imported_count(), 2);
    assert_eq!(summary.duplicate_count(), 0);
    assert!(!summary.dry_run());
    assert_eq!(runtime.register_balance_for("assets:checking"), 98_766);

    let duplicate_summary = runtime
        .import_pdf_statement(&statement_path, "assets:checking", false, false)
        .expect("reimport");
    assert_eq!(duplicate_summary.imported_count(), 0);
    assert_eq!(duplicate_summary.duplicate_count(), 2);

    cleanup_file(Path::new(&statement_path));
}

#[test]
fn e2e_pdf_import_reconciliation_exposes_statement_line_evidence() {
    let path = temp_runtime_path("pdf-reconcile-evidence");
    let statement_path = temp_runtime_path("pdf-reconcile-evidence-statement")
        .with_extension("pdf")
        .to_string_lossy()
        .to_string();
    std::fs::write(&statement_path, "2026-02-01 COFFEE SHOP -12.34\n").expect("write statement");

    let run_id;
    {
        let mut runtime = CliRuntime::open(&path).expect("open");
        let summary = runtime
            .import_pdf_statement(&statement_path, "assets:checking", false, false)
            .expect("import");
        assert_eq!(summary.imported_count(), 1);

        let run = runtime
            .reconcile_and_persist_month_for("assets:checking", "2026-02", 100_000, 98_766)
            .expect("reconcile");
        run_id = run.run_id().to_owned();

        let lines = runtime.statement_lines_for_reconciliation_run(&run_id);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].memo(), "COFFEE SHOP");
        assert_eq!(lines[0].amount_cents(), -1_234);
    }

    {
        let reopened = CliRuntime::open(&path).expect("reopen");
        let lines = reopened.statement_lines_for_reconciliation_run(&run_id);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].memo(), "COFFEE SHOP");
        assert_eq!(lines[0].source_uri(), statement_path);
    }

    cleanup_file(Path::new(&statement_path));
    cleanup_runtime_path(&path);
}

#[test]
fn e2e_pdf_import_dedupe_persists_across_reopen() {
    let root = temp_runtime_path("pdf-reopen-idempotency");
    let ledger_path = root.join("ledger");
    let statement_path = root.join("statement.pdf");
    std::fs::create_dir_all(&root).expect("create root");
    std::fs::write(
        &statement_path,
        "2026-02-01 COFFEE SHOP -12.34\n2026-02-02 PAYROLL 1000.00\n",
    )
    .expect("write statement");

    {
        let mut runtime = CliRuntime::open(&ledger_path).expect("open");
        let summary = runtime
            .import_pdf_statement(&statement_path, "assets:checking", false, false)
            .expect("first import");
        assert_eq!(summary.imported_count(), 2);
        assert_eq!(summary.duplicate_count(), 0);
    }

    {
        let mut reopened = CliRuntime::open(&ledger_path).expect("reopen");
        let summary = reopened
            .import_pdf_statement(&statement_path, "assets:checking", false, false)
            .expect("second import");
        assert_eq!(summary.imported_count(), 0);
        assert_eq!(summary.duplicate_count(), 2);
    }

    cleanup_runtime_path(&root);
}

#[test]
fn e2e_pdf_import_dry_run_does_not_post_transactions() {
    let mut runtime = CliRuntime::new_in_memory();
    let statement_path = temp_runtime_path("pdf-dry-run")
        .with_extension("pdf")
        .to_string_lossy()
        .to_string();
    std::fs::write(&statement_path, "2026-02-01 BOOK STORE -20.00\n").expect("write statement");

    let summary = runtime
        .import_pdf_statement(&statement_path, "assets:checking", true, false)
        .expect("dry-run");
    assert_eq!(summary.imported_count(), 1);
    assert_eq!(summary.duplicate_count(), 0);
    assert!(summary.dry_run());
    assert_eq!(runtime.register_balance_for("assets:checking"), 0);

    cleanup_file(Path::new(&statement_path));
}

#[test]
fn e2e_csv_import_posts_transactions_and_deduplicates() {
    let mut runtime = CliRuntime::new_in_memory();
    let csv_path = temp_runtime_path("csv-import")
        .with_extension("csv")
        .to_string_lossy()
        .to_string();
    std::fs::write(
        &csv_path,
        "timestamp,amount,memo,account,category\n2026-02-01T00:00:00,-1234,COFFEE SHOP,assets:checking,expenses:food\n2026-02-02T00:00:00,100000,PAYROLL,assets:checking,income:salary\n",
    )
    .expect("write csv");
    let mapping = CsvMapping {
        source_id: "checking.csv".to_owned(),
        ..CsvMapping::default()
    };

    let summary = runtime
        .import_csv_statement(&csv_path, &mapping, false, true)
        .expect("csv import");
    assert_eq!(summary.imported_count(), 2);
    assert_eq!(summary.duplicate_count(), 0);
    assert_eq!(runtime.register_balance_for("assets:checking"), 98_766);

    let duplicate_summary = runtime
        .import_csv_statement(&csv_path, &mapping, false, true)
        .expect("csv reimport");
    assert_eq!(duplicate_summary.imported_count(), 0);
    assert_eq!(duplicate_summary.duplicate_count(), 2);

    cleanup_file(Path::new(&csv_path));
}

#[test]
fn e2e_import_backdates_valid_time_from_statement_timestamp() {
    let ledger_path = temp_runtime_path("csv-valid-time-ledger");
    let mut runtime = CliRuntime::open(&ledger_path).expect("open runtime");
    let csv_path = temp_runtime_path("csv-valid-time")
        .with_extension("csv")
        .to_string_lossy()
        .to_string();
    std::fs::write(
        &csv_path,
        "timestamp,amount,memo,account,category\n2026-02-01T00:00:00,-1234,COFFEE SHOP,assets:checking,expenses:food\n",
    )
    .expect("write csv");
    let mapping = CsvMapping::default();

    runtime
        .import_csv_statement(&csv_path, &mapping, false, true)
        .expect("import csv");

    let tx_now_us = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_micros();
    let tx_now_us = i64::try_from(tx_now_us).unwrap_or(i64::MAX);
    let before_valid_us = timestamp_micros("2026-01-31T23:59:59Z");
    let at_valid_us = timestamp_micros("2026-02-01T00:00:00Z");

    let before = runtime
        .transactions_as_of_us(before_valid_us, tx_now_us)
        .expect("as-of before");
    assert!(before.is_empty());

    let at = runtime
        .transactions_as_of_us(at_valid_us, tx_now_us)
        .expect("as-of at");
    assert_eq!(at.len(), 1);

    cleanup_file(Path::new(&csv_path));
    cleanup_runtime_path(&ledger_path);
}

#[test]
fn e2e_month_autopilot_is_atomic_when_close_reference_is_invalid() {
    let mut runtime = CliRuntime::new_in_memory();
    runtime
        .post_double_entry("paycheck", "assets:checking", "income:salary", 10_000)
        .expect("post");
    let request = MonthAutopilotRequest::new("2026-02", "assets:checking", 100_000, 110_000)
        .with_analytics_artifact_id("artifact-missing")
        .with_confirm_close(true)
        .with_allow_variance(true);

    let err = runtime
        .run_month_autopilot(&request)
        .expect_err("autopilot should fail");
    assert!(
        err.to_string().contains("unknown artifact"),
        "expected error containing 'unknown artifact', got: {err}"
    );
    assert_eq!(runtime.reconciliation_run_count(), 0);
    assert!(
        runtime
            .month_close_for_scope("2026-02", "assets:checking")
            .is_none()
    );
}

#[test]
fn e2e_analytics_snapshot_manifest_and_parquet_persist_across_reopen() {
    let root = temp_runtime_path("analytics-snapshot-root");
    let ledger_path = root.join("ledger");
    {
        let mut runtime = CliRuntime::open(&ledger_path).expect("open");
        runtime
            .post_double_entry("paycheck", "assets:checking", "income:salary", 10_000)
            .expect("post");
        let first = runtime
            .create_analytics_snapshot(None, None, 1, None)
            .expect("create first snapshot");

        assert!(Path::new(first.artifact_uri()).exists());
        assert_eq!(first.row_count(), 2);

        let listed = runtime.list_analytics_snapshots();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].artifact_id(), first.artifact_id());

        let loaded = runtime
            .get_analytics_snapshot(first.artifact_id())
            .expect("load snapshot by id");
        assert_eq!(loaded.content_hash(), first.content_hash());

        let second = runtime
            .create_analytics_snapshot(None, None, 1, Some(first.artifact_id()))
            .expect("create second snapshot");
        assert_eq!(second.supersedes_artifact_id(), Some(first.artifact_id()));
    }

    let reopened = CliRuntime::open(&ledger_path).expect("reopen");
    let manifests = reopened.list_analytics_snapshots();
    assert_eq!(manifests.len(), 2);
    assert!(
        manifests
            .iter()
            .any(|manifest| manifest.supersedes_artifact_id().is_some())
    );

    cleanup_runtime_path(&root);
}

#[test]
fn e2e_analytics_snapshot_rejects_non_positive_schema_version() {
    let mut runtime = CliRuntime::new_in_memory();
    let err = runtime
        .create_analytics_snapshot(None, None, 0, None)
        .expect_err("schema version should reject");
    assert_eq!(err.to_string(), "schema_version must be positive, got 0");
}
