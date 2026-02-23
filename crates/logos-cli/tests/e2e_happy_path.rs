use logos_cli::runtime::CliRuntime;
use logos_import::CsvMapping;

#[test]
fn e2e_happy_path_posts_and_reports_register_balance() {
    let mut runtime = CliRuntime::new();

    let txn_id = runtime
        .post_double_entry("paycheck", "assets:checking", "income:salary", 10_000)
        .expect("post");

    assert!(runtime.transaction_exists(&txn_id));
    assert_eq!(runtime.register_balance_for("assets:checking"), 10_000);
}

#[test]
fn e2e_import_is_idempotent_by_fingerprint() {
    let mut runtime = CliRuntime::new();
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
