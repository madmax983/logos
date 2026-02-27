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
        "missing value for argument '--debit-account'"
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
        "missing value for argument '--credit-account'"
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
        "missing value for argument '--amount-cents'"
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
        "invalid value 'ten' for argument '--amount-cents'"
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
fn parses_budget_set_with_default_value_flags() {
    let args = vec!["ledger", "budget", "set"];
    let parsed = logos_cli::parse_args(args).expect("parse");

    assert_eq!(parsed.command_path(), "budget.set");
    assert!(matches!(
        parsed.command(),
        logos_cli::args::Command::Budget(logos_cli::args::BudgetCommand::Set {
            budget_cents,
            expense_account_prefix
        }) if *budget_cents == 0 && expense_account_prefix == "expenses:"
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
        logos_cli::args::Command::Budget(logos_cli::args::BudgetCommand::Set {
            budget_cents,
            expense_account_prefix
        }) if *budget_cents == 250_000 && expense_account_prefix == "expenses:food"
    ));
}

#[test]
fn rejects_budget_set_when_budget_cents_is_not_integer() {
    let args = vec!["ledger", "budget", "set", "--budget-cents", "ten"];
    let err = logos_cli::parse_args(args).expect_err("invalid budget cents");

    assert_eq!(
        err.to_string(),
        "invalid value 'ten' for argument '--budget-cents'"
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
        logos_cli::args::Command::Report(logos_cli::args::ReportCommand::Month {
            checking_account
        }) if checking_account == "assets:checking"
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
        logos_cli::args::Command::Report(logos_cli::args::ReportCommand::Month {
            checking_account
        }) if checking_account == "assets:brokerage"
    ));
}

#[test]
fn parses_aletheia_start_command() {
    let args = vec!["ledger", "aletheia", "start"];
    let parsed = logos_cli::parse_args(args).expect("parse");

    assert_eq!(parsed.command_path(), "aletheia.start");
}

#[test]
fn parses_aletheia_status_command() {
    let args = vec!["ledger", "aletheia", "status"];
    let parsed = logos_cli::parse_args(args).expect("parse");

    assert_eq!(parsed.command_path(), "aletheia.status");
}

#[test]
fn parses_aletheia_help_flag() {
    let args = vec!["ledger", "aletheia", "--help"];
    let parsed = logos_cli::parse_args(args).expect("parse");

    assert_eq!(parsed.command_path(), "help.aletheia");
}

#[test]
fn parses_help_for_aletheia_subcommand() {
    let args = vec!["ledger", "help", "aletheia"];
    let parsed = logos_cli::parse_args(args).expect("parse");

    assert_eq!(parsed.command_path(), "help.aletheia");
}
