use logos_store_pg::{MIGRATIONS, schema};

#[test]
fn migration_sql_mentions_core_tables() {
    let up_sql = include_str!("../migrations/00000000000001_initial_schema/up.sql");

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
            up_sql.contains(&format!("CREATE TABLE {table}")),
            "expected migration to create table '{table}'"
        );
    }
}

#[test]
fn embedded_migrations_and_schema_are_exported() {
    let _ = MIGRATIONS;
    let _ = schema::transactions::table;
    let _ = schema::postings::table;
    let _ = schema::month_closes::table;
}
