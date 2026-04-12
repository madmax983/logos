use logos_store_pg::MIGRATIONS;

#[test]
fn migration_sql_mentions_core_tables() {
    let up_sql = include_str!("../migrations/00000000000001_initial_schema/up.sql");

    assert!(
        up_sql.contains("CREATE SEQUENCE IF NOT EXISTS transaction_id_seq"),
        "expected migration to create transaction id sequence"
    );

    for table in [
        "transactions",
        "postings",
        "corrections",
        "budget_targets",
        "analytics_artifact_manifests",
        "import_batches",
        "import_records",
        "statement_lines",
        "fetch_runs",
        "reconciliation_runs",
        "reconciliation_run_transactions",
        "reconciliation_run_statement_lines",
        "month_closes",
    ] {
        assert!(
            up_sql.contains(&format!("CREATE TABLE IF NOT EXISTS {table}")),
            "expected migration to create table '{table}'"
        );
    }
}

#[test]
fn embedded_migrations_are_exported() {
    let _ = MIGRATIONS;
}
