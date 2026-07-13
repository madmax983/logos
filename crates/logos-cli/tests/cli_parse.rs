#[test]
fn parses_txn_add_command() {
    let args = vec![
        "ledger",
        "txn",
        "add",
        "--description",
        "paycheck",
        "--debit-account",
        "assets:checking",
        "--credit-account",
        "income:salary",
        "--amount-cents",
        "100000",
    ];
    let parsed = logos_cli::parse_args(args).expect("parse");

    assert_eq!(parsed.command_path(), "txn.add");
}

#[test]
fn parses_txn_correct_command() {
    let args = vec![
        "ledger",
        "txn",
        "correct",
        "--supersedes-id",
        "txn-7",
        "--reason",
        "fix memo",
    ];
    let parsed = logos_cli::parse_args(args).expect("parse");

    assert_eq!(parsed.command_path(), "txn.correct");
}

#[test]
fn rejects_txn_correct_when_missing_reason() {
    let args = vec!["ledger", "txn", "correct", "--supersedes-id", "txn-7"];
    let err = logos_cli::parse_args(args).expect_err("missing reason");

    assert_eq!(err.to_string(), "Missing required argument '--reason'.");
}

#[test]
fn rejects_txn_add_when_missing_debit_account_flag() {
    let args = vec![
        "ledger",
        "txn",
        "add",
        "--description",
        "paycheck",
        "--credit-account",
        "income:salary",
        "--amount-cents",
        "100000",
    ];
    let err = logos_cli::parse_args(args).expect_err("missing debit account");

    assert_eq!(
        err.to_string(),
        "Missing required argument '--debit-account'."
    );
}

#[test]
fn rejects_txn_add_when_missing_credit_account_flag() {
    let args = vec![
        "ledger",
        "txn",
        "add",
        "--description",
        "paycheck",
        "--debit-account",
        "assets:checking",
        "--amount-cents",
        "100000",
    ];
    let err = logos_cli::parse_args(args).expect_err("missing credit account");

    assert_eq!(
        err.to_string(),
        "Missing required argument '--credit-account'."
    );
}

#[test]
fn rejects_txn_add_when_missing_amount_cents_flag() {
    let args = vec![
        "ledger",
        "txn",
        "add",
        "--description",
        "paycheck",
        "--debit-account",
        "assets:checking",
        "--credit-account",
        "income:salary",
    ];
    let err = logos_cli::parse_args(args).expect_err("missing amount cents");

    assert_eq!(
        err.to_string(),
        "Missing required argument '--amount-cents'."
    );
}

#[test]
fn rejects_txn_add_when_amount_cents_is_not_integer() {
    let args = vec![
        "ledger",
        "txn",
        "add",
        "--description",
        "paycheck",
        "--debit-account",
        "assets:checking",
        "--credit-account",
        "income:salary",
        "--amount-cents",
        "ten",
    ];
    let err = logos_cli::parse_args(args).expect_err("invalid amount cents");

    assert_eq!(
        err.to_string(),
        "Invalid value 'ten' for argument '--amount-cents'."
    );
}

#[test]
fn parses_help_flag() {
    let args = vec!["ledger", "--help"];
    let parsed = logos_cli::parse_args(args).expect("parse");

    assert_eq!(parsed.command_path(), "help");
}

#[test]
fn parses_short_help_flag() {
    let args = vec!["ledger", "-h"];
    let parsed = logos_cli::parse_args(args).expect("parse");

    assert_eq!(parsed.command_path(), "help");
}

#[test]
fn parses_help_subcommand() {
    let args = vec!["ledger", "help"];
    let parsed = logos_cli::parse_args(args).expect("parse");

    assert_eq!(parsed.command_path(), "help");
}

#[test]
fn runs_help_flag() {
    let args = vec!["ledger", "--help"];

    logos_cli::run(args).expect("run");
}

#[test]
fn parses_help_for_txn_subcommand() {
    let args = vec!["ledger", "help", "txn"];
    let parsed = logos_cli::parse_args(args).expect("parse");

    assert_eq!(parsed.command_path(), "help.txn");
}

#[test]
fn parses_help_for_import_subcommand() {
    let args = vec!["ledger", "help", "import"];
    let parsed = logos_cli::parse_args(args).expect("parse");

    assert_eq!(parsed.command_path(), "help.import");
}

#[test]
fn parses_help_for_analytics_subcommand() {
    let args = vec!["ledger", "help", "analytics"];
    let parsed = logos_cli::parse_args(args).expect("parse");

    assert_eq!(parsed.command_path(), "help.analytics");
}

#[test]
fn parses_help_for_reconcile_subcommand() {
    let args = vec!["ledger", "help", "reconcile"];
    let parsed = logos_cli::parse_args(args).expect("parse");

    assert_eq!(parsed.command_path(), "help.reconcile");
}

#[test]
fn parses_help_for_month_subcommand() {
    let args = vec!["ledger", "help", "month"];
    let parsed = logos_cli::parse_args(args).expect("parse");

    assert_eq!(parsed.command_path(), "help.month");
}

#[test]
fn parses_txn_help_flag() {
    let args = vec!["ledger", "txn", "--help"];
    let parsed = logos_cli::parse_args(args).expect("parse");

    assert_eq!(parsed.command_path(), "help.txn");
}

#[test]
fn parses_budget_help_flag() {
    let args = vec!["ledger", "budget", "--help"];
    let parsed = logos_cli::parse_args(args).expect("parse");

    assert_eq!(parsed.command_path(), "help.budget");
}

#[test]
fn parses_import_help_flag() {
    let args = vec!["ledger", "import", "--help"];
    let parsed = logos_cli::parse_args(args).expect("parse");

    assert_eq!(parsed.command_path(), "help.import");
}

#[test]
fn parses_analytics_help_flag() {
    let args = vec!["ledger", "analytics", "--help"];
    let parsed = logos_cli::parse_args(args).expect("parse");

    assert_eq!(parsed.command_path(), "help.analytics");
}

#[test]
fn parses_reconcile_help_flag() {
    let args = vec!["ledger", "reconcile", "--help"];
    let parsed = logos_cli::parse_args(args).expect("parse");

    assert_eq!(parsed.command_path(), "help.reconcile");
}

#[test]
fn parses_month_help_flag() {
    let args = vec!["ledger", "month", "--help"];
    let parsed = logos_cli::parse_args(args).expect("parse");

    assert_eq!(parsed.command_path(), "help.month");
}

#[test]
fn parses_analytics_snapshot_create_with_defaults() {
    let args = vec!["ledger", "analytics", "snapshot", "create"];
    let parsed = logos_cli::parse_args(args).expect("parse");

    assert_eq!(parsed.command_path(), "analytics.snapshot.create");
    assert!(matches!(
        parsed.command(),
        logos_cli::Command::Analytics(logos_cli::AnalyticsCommand::SnapshotCreate {
            as_of_valid_time_us,
            as_of_tx_time_us,
            schema_version,
            supersedes_artifact_id
        }) if as_of_valid_time_us.is_none()
            && as_of_tx_time_us.is_none()
            && *schema_version == logos_runtime::AppRuntime::<logos_store_pg::PostgresStore>::default_analytics_schema_version()
            && supersedes_artifact_id.is_none()
    ));
}

#[test]
fn parses_analytics_snapshot_create_with_explicit_flags() {
    let args = vec![
        "ledger",
        "analytics",
        "snapshot",
        "create",
        "--as-of-valid-us",
        "1700000000000000",
        "--as-of-tx-us",
        "1700000000000500",
        "--schema-version",
        "3",
        "--supersedes",
        "artifact-9",
    ];
    let parsed = logos_cli::parse_args(args).expect("parse");

    assert!(matches!(
        parsed.command(),
        logos_cli::Command::Analytics(logos_cli::AnalyticsCommand::SnapshotCreate {
            as_of_valid_time_us,
            as_of_tx_time_us,
            schema_version,
            supersedes_artifact_id
        }) if *as_of_valid_time_us == Some(1_700_000_000_000_000)
            && *as_of_tx_time_us == Some(1_700_000_000_000_500)
            && *schema_version == 3
            && supersedes_artifact_id.as_deref() == Some("artifact-9")
    ));
}

#[test]
fn parses_analytics_snapshot_list() {
    let args = vec!["ledger", "analytics", "snapshot", "list"];
    let parsed = logos_cli::parse_args(args).expect("parse");
    assert_eq!(parsed.command_path(), "analytics.snapshot.list");
}

#[test]
fn parses_analytics_net_worth() {
    let args = vec![
        "ledger",
        "analytics",
        "net-worth",
        "--initial-net-worth-cents",
        "10000000",
        "--monthly-savings-cents",
        "500000",
        "--months",
        "12",
    ];
    let parsed = logos_cli::parse_args(args).expect("parses");
    assert_eq!(parsed.command_path(), "analytics.net-worth");
    assert_eq!(
        parsed.command(),
        &logos_cli::Command::Analytics(logos_cli::AnalyticsCommand::NetWorthProject {
            initial_net_worth_cents: 10_000_000,
            monthly_savings_cents: 500_000,
            months: 12,
        })
    );
}

#[test]
fn parses_analytics_sankey() {
    let args = vec!["ledger", "analytics", "sankey"];
    let parsed = logos_cli::parse_args(args).expect("parse");
    assert_eq!(parsed.command_path(), "analytics.sankey");
}

#[test]
fn parses_analytics_snapshot_show() {
    let args = vec![
        "ledger",
        "analytics",
        "snapshot",
        "show",
        "--artifact-id",
        "artifact-7",
    ];
    let parsed = logos_cli::parse_args(args).expect("parse");
    assert_eq!(parsed.command_path(), "analytics.snapshot.show");
}

#[test]
fn rejects_analytics_snapshot_show_without_artifact_id() {
    let args = vec!["ledger", "analytics", "snapshot", "show"];
    let err = logos_cli::parse_args(args).expect_err("missing artifact id");
    assert_eq!(
        err.to_string(),
        "Missing required argument '--artifact-id'."
    );
}

#[test]
fn parses_import_pdf_with_default_value_flags() {
    let args = vec!["ledger", "import", "pdf", "--file", "statement.pdf"];
    let parsed = logos_cli::parse_args(args).expect("parse");

    assert_eq!(parsed.command_path(), "import.pdf");
    assert!(matches!(
        parsed.command(),
        logos_cli::Command::Import(logos_cli::ImportCommand::Pdf {
            file_path,
            account,
            dry_run,
            ocr
        }) if file_path == "statement.pdf"
            && account == "assets:checking"
            && !*dry_run
            && !*ocr
    ));
}

#[test]
fn parses_import_pdf_with_explicit_value_flags() {
    let args = vec![
        "ledger",
        "import",
        "pdf",
        "--file",
        "statement.pdf",
        "--account",
        "assets:brokerage",
        "--dry-run",
    ];
    let parsed = logos_cli::parse_args(args).expect("parse");

    assert_eq!(parsed.command_path(), "import.pdf");
    assert!(matches!(
        parsed.command(),
        logos_cli::Command::Import(logos_cli::ImportCommand::Pdf {
            file_path,
            account,
            dry_run,
            ocr
        }) if file_path == "statement.pdf"
            && account == "assets:brokerage"
            && *dry_run
            && !*ocr
    ));
}

#[test]
fn parses_import_pdf_with_ocr_flag() {
    let args = vec![
        "ledger",
        "import",
        "pdf",
        "--file",
        "statement.pdf",
        "--ocr",
    ];
    let parsed = logos_cli::parse_args(args).expect("parse");

    assert_eq!(parsed.command_path(), "import.pdf");
    assert!(matches!(
        parsed.command(),
        logos_cli::Command::Import(logos_cli::ImportCommand::Pdf {
            file_path,
            account,
            dry_run,
            ocr
        }) if file_path == "statement.pdf"
            && account == "assets:checking"
            && !*dry_run
            && *ocr
    ));
}

#[test]
fn rejects_import_pdf_when_missing_file_flag() {
    let args = vec!["ledger", "import", "pdf"];
    let err = logos_cli::parse_args(args).expect_err("missing file flag");

    assert_eq!(err.to_string(), "Missing required argument '--file'.");
}

#[test]
fn parses_import_csv_with_defaults() {
    let args = vec!["ledger", "import", "csv", "--file", "statement.csv"];
    let parsed = logos_cli::parse_args(args).expect("parse");

    assert_eq!(parsed.command_path(), "import.csv");
    assert!(matches!(
        parsed.command(),
        logos_cli::Command::Import(logos_cli::ImportCommand::Csv {
            file_path,
            source_id,
            timestamp_idx,
            amount_idx,
            memo_idx,
            account_idx,
            category_idx,
            skip_header,
            dry_run,
        }) if file_path == "statement.csv"
            && source_id.is_none()
            && *timestamp_idx == 0
            && *amount_idx == 1
            && *memo_idx == 2
            && *account_idx == 3
            && *category_idx == 4
            && !*skip_header
            && !*dry_run
    ));
}

#[test]
fn parses_import_csv_with_explicit_mapping_flags() {
    let args = vec![
        "ledger",
        "import",
        "csv",
        "--file",
        "statement.csv",
        "--source-id",
        "chase.csv",
        "--timestamp-idx",
        "1",
        "--amount-idx",
        "3",
        "--memo-idx",
        "4",
        "--account-idx",
        "6",
        "--category-idx",
        "7",
        "--skip-header",
        "--dry-run",
    ];
    let parsed = logos_cli::parse_args(args).expect("parse");

    assert!(matches!(
        parsed.command(),
        logos_cli::Command::Import(logos_cli::ImportCommand::Csv {
            file_path,
            source_id,
            timestamp_idx,
            amount_idx,
            memo_idx,
            account_idx,
            category_idx,
            skip_header,
            dry_run,
        }) if file_path == "statement.csv"
            && source_id.as_deref() == Some("chase.csv")
            && *timestamp_idx == 1
            && *amount_idx == 3
            && *memo_idx == 4
            && *account_idx == 6
            && *category_idx == 7
            && *skip_header
            && *dry_run
    ));
}

#[test]
fn rejects_import_csv_when_missing_required_flag_value() {
    let args = vec![
        "ledger".to_string(),
        "import".to_string(),
        "csv".to_string(),
        "--file".to_string(),
        "--timestamp-idx".to_string(),
    ];
    let err = logos_cli::parse_args(args).expect_err("missing file flag value");
    assert_eq!(err.to_string(), "Missing value for argument '--file'.");
}

#[test]
fn rejects_budget_set_when_missing_optional_flag_value() {
    let args = vec![
        "ledger".to_string(),
        "budget".to_string(),
        "set".to_string(),
        "--expense-account-prefix".to_string(),
        "--budget-cents".to_string(),
    ];
    let err = logos_cli::parse_args(args).expect_err("missing account flag value");
    // Since `--budget-cents` is parsed first in the source, it is the one missing its value!
    assert_eq!(
        err.to_string(),
        "Missing value for argument '--budget-cents'."
    );
}

#[test]
fn rejects_import_csv_when_mapping_index_is_not_integer() {
    let args = vec![
        "ledger",
        "import",
        "csv",
        "--file",
        "statement.csv",
        "--amount-idx",
        "x",
    ];
    let err = logos_cli::parse_args(args).expect_err("invalid csv amount idx");

    assert_eq!(
        err.to_string(),
        "Invalid value 'x' for argument '--amount-idx'."
    );
}

#[test]
fn parses_budget_set_with_default_value_flags() {
    let args = vec!["ledger", "budget", "set"];
    let parsed = logos_cli::parse_args(args).expect("parse");

    assert_eq!(parsed.command_path(), "budget.set");
    assert!(matches!(
        parsed.command(),
        logos_cli::Command::Budget(logos_cli::BudgetCommand::Set {
            month_key,
            budget_cents,
            expense_account_prefix
        }) if month_key.is_none() && *budget_cents == 0 && expense_account_prefix == "expenses:"
    ));
}

#[test]
fn parses_budget_set_with_explicit_value_flags() {
    let args = vec![
        "ledger",
        "budget",
        "set",
        "--budget-cents",
        "250000",
        "--expense-account-prefix",
        "expenses:food",
    ];
    let parsed = logos_cli::parse_args(args).expect("parse");

    assert_eq!(parsed.command_path(), "budget.set");
    assert!(matches!(
        parsed.command(),
        logos_cli::Command::Budget(logos_cli::BudgetCommand::Set {
            month_key,
            budget_cents,
            expense_account_prefix
        }) if month_key.is_none() && *budget_cents == 250_000 && expense_account_prefix == "expenses:food"
    ));
}

#[test]
fn rejects_budget_set_when_budget_cents_is_not_integer() {
    let args = vec!["ledger", "budget", "set", "--budget-cents", "ten"];
    let err = logos_cli::parse_args(args).expect_err("invalid budget cents");

    assert_eq!(
        err.to_string(),
        "Invalid value 'ten' for argument '--budget-cents'."
    );
}

#[test]
fn parses_budget_set_with_month_flag() {
    let args = vec!["ledger", "budget", "set", "--month", "2026-03"];
    let parsed = logos_cli::parse_args(args).expect("parse");

    assert!(matches!(
        parsed.command(),
        logos_cli::Command::Budget(logos_cli::BudgetCommand::Set {
            month_key,
            budget_cents,
            expense_account_prefix
        }) if month_key.as_deref() == Some("2026-03")
            && *budget_cents == 0
            && expense_account_prefix == "expenses:"
    ));
}

#[test]
fn rejects_budget_set_when_month_is_invalid() {
    let args = vec!["ledger", "budget", "set", "--month", "2026-13"];
    let err = logos_cli::parse_args(args).expect_err("invalid month");

    assert_eq!(
        err.to_string(),
        "Invalid value '2026-13' for argument '--month'."
    );
}

#[test]
fn parses_budget_rsu_plan_with_defaults() {
    let args = vec![
        "ledger",
        "budget",
        "rsu-plan",
        "--quarterly-units",
        "300",
        "--bear-price-cents",
        "10000",
        "--base-price-cents",
        "12000",
        "--bull-price-cents",
        "16000",
    ];
    let parsed = logos_cli::parse_args(args).expect("parse");

    assert_eq!(parsed.command_path(), "budget.rsu-plan");
    assert!(matches!(
        parsed.command(),
        logos_cli::Command::Budget(logos_cli::BudgetCommand::RsuPlan {
            month_key,
            quarterly_units,
            days_to_vest,
            bear_price_cents,
            base_price_cents,
            bull_price_cents,
            fixed_commitments_cents,
            reserve_sweep_pct,
            investing_sweep_pct,
        }) if month_key.is_none()
            && *quarterly_units == 300
            && *days_to_vest == 45
            && *bear_price_cents == 10_000
            && *base_price_cents == 12_000
            && *bull_price_cents == 16_000
            && *fixed_commitments_cents == 0
            && *reserve_sweep_pct == 60
            && *investing_sweep_pct == 30
    ));
}

#[test]
fn rejects_budget_rsu_plan_without_quarterly_units() {
    let args = vec![
        "ledger",
        "budget",
        "rsu-plan",
        "--bear-price-cents",
        "10000",
        "--base-price-cents",
        "12000",
        "--bull-price-cents",
        "16000",
    ];
    let err = logos_cli::parse_args(args).expect_err("missing units");

    assert_eq!(
        err.to_string(),
        "Missing required argument '--quarterly-units'."
    );
}

#[test]
fn parses_report_help_flag() {
    let args = vec!["ledger", "report", "--help"];
    let parsed = logos_cli::parse_args(args).expect("parse");

    assert_eq!(parsed.command_path(), "help.report");
}

#[test]
fn parses_report_month_with_default_value_flags() {
    let args = vec!["ledger", "report", "month"];
    let parsed = logos_cli::parse_args(args).expect("parse");

    assert_eq!(parsed.command_path(), "report.month");
    assert!(matches!(
        parsed.command(),
        logos_cli::Command::Report(logos_cli::ReportCommand::Month {
            checking_account,
            month_key
        }) if checking_account == "assets:checking" && month_key.is_none()
    ));
}

#[test]
fn parses_report_month_with_explicit_checking_account() {
    let args = vec![
        "ledger",
        "report",
        "month",
        "--checking-account",
        "assets:brokerage",
    ];
    let parsed = logos_cli::parse_args(args).expect("parse");

    assert_eq!(parsed.command_path(), "report.month");
    assert!(matches!(
        parsed.command(),
        logos_cli::Command::Report(logos_cli::ReportCommand::Month {
            checking_account,
            month_key
        }) if checking_account == "assets:brokerage" && month_key.is_none()
    ));
}

#[test]
fn parses_report_month_with_month_flag() {
    let args = vec!["ledger", "report", "month", "--month", "2026-04"];
    let parsed = logos_cli::parse_args(args).expect("parse");

    assert!(matches!(
        parsed.command(),
        logos_cli::Command::Report(logos_cli::ReportCommand::Month {
            checking_account,
            month_key
        }) if checking_account == "assets:checking" && month_key.as_deref() == Some("2026-04")
    ));
}

#[test]
fn rejects_report_month_when_month_is_invalid() {
    let args = vec!["ledger", "report", "month", "--month", "2026-00"];
    let err = logos_cli::parse_args(args).expect_err("invalid month");

    assert_eq!(
        err.to_string(),
        "Invalid value '2026-00' for argument '--month'."
    );
}

#[test]
fn parses_reconcile_month_with_defaults() {
    let args = vec![
        "ledger",
        "reconcile",
        "month",
        "--opening-balance-cents",
        "100000",
        "--closing-balance-cents",
        "107500",
    ];
    let parsed = logos_cli::parse_args(args).expect("parse");

    assert_eq!(parsed.command_path(), "reconcile.month");
    assert!(matches!(
        parsed.command(),
        logos_cli::Command::Reconcile(logos_cli::ReconcileCommand::Month {
            checking_account,
            month_key,
            opening_balance_cents,
            closing_balance_cents
        }) if checking_account == "assets:checking"
            && month_key.is_none()
            && *opening_balance_cents == 100_000
            && *closing_balance_cents == 107_500
    ));
}

#[test]
fn parses_reconcile_month_with_explicit_value_flags() {
    let args = vec![
        "ledger",
        "reconcile",
        "month",
        "--month",
        "2026-04",
        "--checking-account",
        "assets:brokerage",
        "--opening-balance-cents",
        "250000",
        "--closing-balance-cents",
        "260500",
    ];
    let parsed = logos_cli::parse_args(args).expect("parse");

    assert_eq!(parsed.command_path(), "reconcile.month");
    assert!(matches!(
        parsed.command(),
        logos_cli::Command::Reconcile(logos_cli::ReconcileCommand::Month {
            checking_account,
            month_key,
            opening_balance_cents,
            closing_balance_cents
        }) if checking_account == "assets:brokerage"
            && month_key.as_deref() == Some("2026-04")
            && *opening_balance_cents == 250_000
            && *closing_balance_cents == 260_500
    ));
}

#[test]
fn rejects_reconcile_month_when_missing_opening_balance() {
    let args = vec![
        "ledger",
        "reconcile",
        "month",
        "--closing-balance-cents",
        "100000",
    ];
    let err = logos_cli::parse_args(args).expect_err("missing opening balance");

    assert_eq!(
        err.to_string(),
        "Missing required argument '--opening-balance-cents'."
    );
}

#[test]
fn rejects_reconcile_month_when_closing_balance_is_not_integer() {
    let args = vec![
        "ledger",
        "reconcile",
        "month",
        "--opening-balance-cents",
        "100000",
        "--closing-balance-cents",
        "ten",
    ];
    let err = logos_cli::parse_args(args).expect_err("invalid closing balance");

    assert_eq!(
        err.to_string(),
        "Invalid value 'ten' for argument '--closing-balance-cents'."
    );
}

#[test]
fn parses_reconcile_list_with_optional_filters() {
    let args = vec![
        "ledger",
        "reconcile",
        "list",
        "--month",
        "2026-04",
        "--checking-account",
        "assets:brokerage",
    ];
    let parsed = logos_cli::parse_args(args).expect("parse");

    assert_eq!(parsed.command_path(), "reconcile.list");
    assert!(matches!(
        parsed.command(),
        logos_cli::Command::Reconcile(logos_cli::ReconcileCommand::List {
            month_key,
            checking_account,
        }) if month_key.as_deref() == Some("2026-04")
            && checking_account.as_deref() == Some("assets:brokerage")
    ));
}

#[test]
fn parses_reconcile_list_without_filters() {
    let args = vec!["ledger", "reconcile", "list"];
    let parsed = logos_cli::parse_args(args).expect("parse");

    assert_eq!(parsed.command_path(), "reconcile.list");
    assert!(matches!(
        parsed.command(),
        logos_cli::Command::Reconcile(logos_cli::ReconcileCommand::List {
            month_key,
            checking_account,
        }) if month_key.is_none() && checking_account.is_none()
    ));
}

#[test]
fn parses_reconcile_show_with_run_id() {
    let args = vec!["ledger", "reconcile", "show", "--run-id", "recon-17"];
    let parsed = logos_cli::parse_args(args).expect("parse");

    assert_eq!(parsed.command_path(), "reconcile.show");
    assert!(matches!(
        parsed.command(),
        logos_cli::Command::Reconcile(logos_cli::ReconcileCommand::Show { run_id })
            if run_id == "recon-17"
    ));
}

#[test]
fn rejects_reconcile_show_without_run_id() {
    let args = vec!["ledger", "reconcile", "show"];
    let err = logos_cli::parse_args(args).expect_err("missing run id");

    assert_eq!(err.to_string(), "Missing required argument '--run-id'.");
}

#[test]
fn parses_fetch_list_runs_with_optional_filters() {
    let args = vec![
        "ledger",
        "fetch",
        "list-runs",
        "--month",
        "2026-04",
        "--checking-account",
        "assets:checking",
    ];
    let parsed = logos_cli::parse_args(args).expect("parse");

    assert_eq!(parsed.command_path(), "fetch.list");
    assert!(matches!(
        parsed.command(),
        logos_cli::Command::Fetch(logos_cli::FetchCommand::ListRuns {
            month_key,
            checking_account,
        }) if month_key.as_deref() == Some("2026-04")
            && checking_account.as_deref() == Some("assets:checking")
    ));
}

#[test]
fn parses_fetch_show_with_run_id() {
    let args = vec!["ledger", "fetch", "show-run", "--run-id", "fetch-17"];
    let parsed = logos_cli::parse_args(args).expect("parse");

    assert_eq!(parsed.command_path(), "fetch.show");
    assert!(matches!(
        parsed.command(),
        logos_cli::Command::Fetch(logos_cli::FetchCommand::ShowRun { run_id })
            if run_id == "fetch-17"
    ));
}

#[test]
fn rejects_fetch_show_without_run_id() {
    let args = vec!["ledger", "fetch", "show-run"];
    let err = logos_cli::parse_args(args).expect_err("missing run id");

    assert_eq!(err.to_string(), "Missing required argument '--run-id'.");
}

#[test]
fn parses_month_autopilot_with_defaults() {
    let args = vec!["ledger", "month", "autopilot", "--confirm-close"];
    let parsed = logos_cli::parse_args(args).expect("parse");

    assert_eq!(parsed.command_path(), "month.autopilot");
    assert!(matches!(
        parsed.command(),
        logos_cli::Command::Month(logos_cli::MonthCommand::Autopilot {
            month_key,
            checking_account,
            opening_balance_cents,
            closing_balance_cents,
            statement_pdf,
            ocr,
            allow_variance,
            analytics_artifact_id,
            confirm_close,
        }) if month_key.is_none()
            && checking_account == "assets:checking"
            && opening_balance_cents.is_none()
            && closing_balance_cents.is_none()
            && statement_pdf.is_none()
            && !*ocr
            && !*allow_variance
            && analytics_artifact_id.is_none()
            && *confirm_close
    ));
}

#[test]
fn parses_month_autopilot_with_explicit_flags() {
    let args = vec![
        "ledger",
        "month",
        "autopilot",
        "--month",
        "2026-04",
        "--checking-account",
        "assets:brokerage",
        "--opening-balance-cents",
        "250000",
        "--closing-balance-cents",
        "260500",
        "--statement-pdf",
        "statement.pdf",
        "--ocr",
        "--allow-variance",
        "--analytics-artifact-id",
        "artifact-7",
        "--confirm-close",
    ];
    let parsed = logos_cli::parse_args(args).expect("parse");

    assert!(matches!(
        parsed.command(),
        logos_cli::Command::Month(logos_cli::MonthCommand::Autopilot {
            month_key,
            checking_account,
            opening_balance_cents,
            closing_balance_cents,
            statement_pdf,
            ocr,
            allow_variance,
            analytics_artifact_id,
            confirm_close,
        }) if month_key.as_deref() == Some("2026-04")
            && checking_account == "assets:brokerage"
            && opening_balance_cents == &Some(250_000)
            && closing_balance_cents == &Some(260_500)
            && statement_pdf.as_deref() == Some("statement.pdf")
            && *ocr
            && *allow_variance
            && analytics_artifact_id.as_deref() == Some("artifact-7")
            && *confirm_close
    ));
}

#[test]
fn rejects_month_autopilot_when_only_one_balance_flag_is_present() {
    let args = vec![
        "ledger",
        "month",
        "autopilot",
        "--closing-balance-cents",
        "107500",
        "--confirm-close",
    ];
    let err = logos_cli::parse_args(args).expect_err("mismatched balance flags");

    assert_eq!(
        err.to_string(),
        "Missing required argument '--opening-balance-cents'."
    );
}

#[test]
fn parses_close_month_with_required_run_id() {
    let args = vec!["ledger", "close", "month", "--run-id", "recon-9"];
    let parsed = logos_cli::parse_args(args).expect("parse");

    assert_eq!(parsed.command_path(), "close.month");
    assert!(matches!(
        parsed.command(),
        logos_cli::Command::Close(logos_cli::CloseCommand::Month {
            month_key,
            checking_account,
            run_id,
            analytics_artifact_id
        }) if month_key.is_none()
            && checking_account == "assets:checking"
            && run_id == "recon-9"
            && analytics_artifact_id.is_none()
    ));
}

#[test]
fn parses_close_month_with_explicit_flags() {
    let args = vec![
        "ledger",
        "close",
        "month",
        "--month",
        "2026-04",
        "--checking-account",
        "assets:brokerage",
        "--run-id",
        "recon-11",
        "--analytics-artifact-id",
        "artifact-7",
    ];
    let parsed = logos_cli::parse_args(args).expect("parse");

    assert_eq!(parsed.command_path(), "close.month");
    assert!(matches!(
        parsed.command(),
        logos_cli::Command::Close(logos_cli::CloseCommand::Month {
            month_key,
            checking_account,
            run_id,
            analytics_artifact_id
        }) if month_key.as_deref() == Some("2026-04")
            && checking_account == "assets:brokerage"
            && run_id == "recon-11"
            && analytics_artifact_id.as_deref() == Some("artifact-7")
    ));
}

#[test]
fn rejects_close_month_without_run_id() {
    let args = vec!["ledger", "close", "month"];
    let err = logos_cli::parse_args(args).expect_err("missing run id");

    assert_eq!(err.to_string(), "Missing required argument '--run-id'.");
}

#[test]
fn parses_db_migrate_command() {
    let args = vec!["ledger", "db", "migrate"];
    let parsed = logos_cli::parse_args(args).expect("parse");

    assert_eq!(parsed.command_path(), "db.migrate");
}

#[test]
fn parses_db_status_command() {
    let args = vec!["ledger", "db", "status"];
    let parsed = logos_cli::parse_args(args).expect("parse");

    assert_eq!(parsed.command_path(), "db.status");
}

#[test]
fn parses_db_help_flag() {
    let args = vec!["ledger", "db", "--help"];
    let parsed = logos_cli::parse_args(args).expect("parse");

    assert_eq!(parsed.command_path(), "help.db");
}

#[test]
fn parses_help_for_db_subcommand() {
    let args = vec!["ledger", "help", "db"];
    let parsed = logos_cli::parse_args(args).expect("parse");

    assert_eq!(parsed.command_path(), "help.db");
}

#[test]
fn rejects_aletheia_command_after_cutover() {
    let args = vec!["ledger", "aletheia", "status"];
    let err = logos_cli::parse_args(args).expect_err("aletheia must be gone");

    assert_eq!(err.to_string(), "Unknown command 'aletheia'.");
}

#[test]
fn parses_budget_monte_carlo() {
    let args = vec![
        "ledger",
        "budget",
        "monte-carlo",
        "--quarterly-units",
        "100",
        "--vest-price",
        "50",
        "--initial-price",
        "50",
        "--volatility",
        "0.2",
        "--initial-cents",
        "1000",
        "--years",
        "10",
        "--monthly-contribution-cents",
        "100",
        "--annual-mean-return",
        "0.08",
        "--annual-volatility",
        "0.15",
        "--months",
        "12",
        "--paths",
        "1000",
    ];
    let parsed = logos_cli::parse_args(args).expect("parse");
    assert_eq!(parsed.command_path(), "budget.monte-carlo");
    assert!(matches!(
        parsed.command(),
        logos_cli::Command::Budget(logos_cli::BudgetCommand::MonteCarlo { .. })
    ));
}

#[test]
fn parses_analytics_fire_sim() {
    let args = vec![
        "ledger",
        "analytics",
        "fire-sim",
        "--initial-net-worth-cents",
        "1000",
        "--monthly-savings-cents",
        "100",
        "--monthly-savings-growth-cents",
        "5",
        "--market-growth-rate",
        "1.05",
        "--months",
        "12",
        "--monte-carlo-iterations",
        "1000",
        "--monthly-expenses-cents",
        "50",
        "--liquid-assets-cents",
        "500",
    ];
    let parsed = logos_cli::parse_args(args).expect("parse");
    assert_eq!(parsed.command_path(), "analytics.fire-sim");
    assert!(matches!(
        parsed.command(),
        logos_cli::Command::Analytics(logos_cli::AnalyticsCommand::FireSim { .. })
    ));
}

#[test]
fn test_parse_month_key_invalid_length() {
    let args = vec!["ledger", "reconcile", "month", "--month", "2026-4"];
    let err = logos_cli::parse_args(args).unwrap_err();
    assert_eq!(
        err.to_string(),
        "Invalid value '2026-4' for argument '--month'."
    );
}

#[test]
fn test_parse_month_key_missing_dash() {
    let args = vec!["ledger", "reconcile", "month", "--month", "2026004"];
    let err = logos_cli::parse_args(args).unwrap_err();
    assert_eq!(
        err.to_string(),
        "Invalid value '2026004' for argument '--month'."
    );
}

#[test]
fn test_parse_month_key_invalid_year_digits() {
    let args = vec!["ledger", "reconcile", "month", "--month", "abcd-04"];
    let err = logos_cli::parse_args(args).unwrap_err();
    assert_eq!(
        err.to_string(),
        "Invalid value 'abcd-04' for argument '--month'."
    );
}

#[test]
fn test_parse_month_key_invalid_month_digits() {
    let args = vec!["ledger", "reconcile", "month", "--month", "2026-ab"];
    let err = logos_cli::parse_args(args).unwrap_err();
    assert_eq!(
        err.to_string(),
        "Invalid value '2026-ab' for argument '--month'."
    );
}

#[test]
fn parses_txn_help_flag_alone() {
    let args = vec!["ledger", "txn", "-h"];
    let parsed = logos_cli::parse_args(args).expect("parse");
    assert_eq!(parsed.command_path(), "help.txn");
}

#[test]
fn parses_txn_help_subcommand() {
    let args = vec!["ledger", "txn", "--help"];
    let parsed = logos_cli::parse_args(args).expect("parse");
    assert_eq!(parsed.command_path(), "help.txn");
}

#[test]
fn test_parse_txn_missing_subcommand_but_has_help_flag() {
    let args = vec!["ledger", "txn", "-h"];
    let parsed = logos_cli::parse_args(args).expect("parse");
    assert_eq!(parsed.command_path(), "help.txn");
}

#[test]
fn test_parse_budget_help_subcommand() {
    let args = vec!["ledger", "budget", "--help"];
    let parsed = logos_cli::parse_args(args).unwrap();
    assert_eq!(parsed.command_path(), "help.budget");
}

#[test]
fn test_parse_analytics_help_subcommand() {
    let args = vec!["ledger", "analytics", "--help"];
    let parsed = logos_cli::parse_args(args).unwrap();
    assert_eq!(parsed.command_path(), "help.analytics");
}

#[test]
fn test_parse_analytics_snapshot_help_subcommand() {
    let args = vec!["ledger", "analytics", "snapshot", "--help"];
    let parsed = logos_cli::parse_args(args).unwrap();
    assert_eq!(parsed.command_path(), "help.analytics");
}

#[test]
fn test_parse_import_help_subcommand() {
    let args = vec!["ledger", "import", "--help"];
    let parsed = logos_cli::parse_args(args).unwrap();
    assert_eq!(parsed.command_path(), "help.import");
}

#[test]
fn test_parse_fetch_help_subcommand() {
    let args = vec!["ledger", "fetch", "--help"];
    let parsed = logos_cli::parse_args(args).unwrap();
    assert_eq!(parsed.command_path(), "help.fetch");
}

#[test]
fn test_parse_report_help_subcommand() {
    let args = vec!["ledger", "report", "--help"];
    let parsed = logos_cli::parse_args(args).unwrap();
    assert_eq!(parsed.command_path(), "help.report");
}

#[test]
fn test_parse_reconcile_help_subcommand() {
    let args = vec!["ledger", "reconcile", "--help"];
    let parsed = logos_cli::parse_args(args).unwrap();
    assert_eq!(parsed.command_path(), "help.reconcile");
}

#[test]
fn test_parse_month_help_subcommand() {
    let args = vec!["ledger", "month", "--help"];
    let parsed = logos_cli::parse_args(args).unwrap();
    assert_eq!(parsed.command_path(), "help.month");
}

#[test]
fn test_parse_close_help_subcommand() {
    let args = vec!["ledger", "close", "--help"];
    let parsed = logos_cli::parse_args(args).unwrap();
    assert_eq!(parsed.command_path(), "help.close");
}

#[test]
fn test_parse_db_help_subcommand() {
    let args = vec!["ledger", "db", "--help"];
    let parsed = logos_cli::parse_args(args).unwrap();
    assert_eq!(parsed.command_path(), "help.db");
}

#[test]
fn test_parse_txn_help_flag_alias() {
    let args = vec!["ledger", "txn", "-h"];
    let parsed = logos_cli::parse_args(args).unwrap();
    assert_eq!(parsed.command_path(), "help.txn");
}

#[test]
fn test_parse_budget_help_flag_alias() {
    let args = vec!["ledger", "budget", "-h"];
    let parsed = logos_cli::parse_args(args).unwrap();
    assert_eq!(parsed.command_path(), "help.budget");
}

#[test]
fn test_parse_analytics_help_flag_alias() {
    let args = vec!["ledger", "analytics", "-h"];
    let parsed = logos_cli::parse_args(args).unwrap();
    assert_eq!(parsed.command_path(), "help.analytics");
}

#[test]
fn test_parse_analytics_snapshot_help_flag_alias() {
    let args = vec!["ledger", "analytics", "snapshot", "-h"];
    let parsed = logos_cli::parse_args(args).unwrap();
    assert_eq!(parsed.command_path(), "help.analytics");
}

#[test]
fn test_parse_import_help_flag_alias() {
    let args = vec!["ledger", "import", "-h"];
    let parsed = logos_cli::parse_args(args).unwrap();
    assert_eq!(parsed.command_path(), "help.import");
}

#[test]
fn test_parse_fetch_help_flag_alias() {
    let args = vec!["ledger", "fetch", "-h"];
    let parsed = logos_cli::parse_args(args).unwrap();
    assert_eq!(parsed.command_path(), "help.fetch");
}

#[test]
fn test_parse_report_help_flag_alias() {
    let args = vec!["ledger", "report", "-h"];
    let parsed = logos_cli::parse_args(args).unwrap();
    assert_eq!(parsed.command_path(), "help.report");
}

#[test]
fn test_parse_reconcile_help_flag_alias() {
    let args = vec!["ledger", "reconcile", "-h"];
    let parsed = logos_cli::parse_args(args).unwrap();
    assert_eq!(parsed.command_path(), "help.reconcile");
}

#[test]
fn test_parse_month_help_flag_alias() {
    let args = vec!["ledger", "month", "-h"];
    let parsed = logos_cli::parse_args(args).unwrap();
    assert_eq!(parsed.command_path(), "help.month");
}

#[test]
fn test_parse_close_help_flag_alias() {
    let args = vec!["ledger", "close", "-h"];
    let parsed = logos_cli::parse_args(args).unwrap();
    assert_eq!(parsed.command_path(), "help.close");
}

#[test]
fn test_parse_db_help_flag_alias() {
    let args = vec!["ledger", "db", "-h"];
    let parsed = logos_cli::parse_args(args).unwrap();
    assert_eq!(parsed.command_path(), "help.db");
}
