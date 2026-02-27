use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use logos_cli::runtime::CliRuntime;
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

    runtime
        .post_double_entry("paycheck", "assets:checking", "income:salary", 10_000)
        .expect("post income");
    runtime
        .post_double_entry("groceries", "expenses:food", "assets:checking", 2_500)
        .expect("post expense");

    let report = runtime.month_report_for("assets:checking");
    assert_eq!(report.checking_balance_cents(), 7_500);
    assert_eq!(report.income_cents(), 10_000);
    assert_eq!(report.expense_cents(), 2_500);
    assert_eq!(report.cashflow_cents(), 7_500);
    assert_eq!(runtime.budget_variance_for(3_000, "expenses:"), 500);
}
