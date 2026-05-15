#[test]
fn test_execute_propagates_errors_from_handlers() {
    let parsed_args_str = vec!["logos-cli", "fetch", "show-run", "--run-id", "nonexistent"];
    let parsed = logos_cli::parse_args(parsed_args_str).unwrap();

    // Now execute it. It should attempt to fetch a nonexistent run and fail.
    assert!(parsed.execute().is_err());
}

#[test]
fn test_execute_propagates_success_from_handlers() {
    let parsed_args_str = vec!["logos-cli", "help"];
    let parsed = logos_cli::parse_args(parsed_args_str).unwrap();

    // Now execute it. Help returns success.
    assert!(parsed.execute().is_ok());
}

#[test]
fn test_execute_propagates_db_error() {
    let parsed_args_str = vec!["logos-cli", "db", "status"];
    let parsed = logos_cli::parse_args(parsed_args_str).unwrap();
    assert!(parsed.execute().is_err());
}

#[test]
fn test_execute_propagates_txn_error() {
    let parsed_args_str = vec!["logos-cli", "txn", "correct", "--supersedes-id", "nonexistent", "--reason", "test"];
    let parsed = logos_cli::parse_args(parsed_args_str).unwrap();
    assert!(parsed.execute().is_err());
}

#[test]
fn test_execute_propagates_analytics_error() {
    let parsed_args_str = vec!["logos-cli", "analytics", "snapshot", "show", "--artifact-id", "nonexistent"];
    let parsed = logos_cli::parse_args(parsed_args_str).unwrap();
    assert!(parsed.execute().is_err());
}

#[test]
fn test_execute_propagates_import_error() {
    let parsed_args_str = vec!["logos-cli", "import", "pdf", "--file", "nonexistent.pdf"];
    let parsed = logos_cli::parse_args(parsed_args_str).unwrap();
    assert!(parsed.execute().is_err());
}

#[test]
fn test_execute_propagates_reconcile_error() {
    let parsed_args_str = vec!["logos-cli", "reconcile", "show", "--run-id", "nonexistent"];
    let parsed = logos_cli::parse_args(parsed_args_str).unwrap();
    assert!(parsed.execute().is_err());
}

#[test]
fn test_execute_propagates_month_error() {
    let parsed_args_str = vec!["logos-cli", "month", "autopilot"];
    let parsed = logos_cli::parse_args(parsed_args_str).unwrap();
    assert!(parsed.execute().is_err());
}

#[test]
fn test_execute_propagates_close_error() {
    let parsed_args_str = vec!["logos-cli", "close", "month", "--run-id", "nonexistent"];
    let parsed = logos_cli::parse_args(parsed_args_str).unwrap();
    assert!(parsed.execute().is_err());
}

#[test]
fn test_execute_propagates_budget_error() {
    let parsed_args_str = vec!["logos-cli", "budget", "rsu-plan", "--quarterly-units", "10", "--bear-price-cents", "100", "--base-price-cents", "200", "--bull-price-cents", "300"];
    let parsed = logos_cli::parse_args(parsed_args_str).unwrap();
    assert!(parsed.execute().is_err());
}

#[test]
fn test_execute_propagates_report_error() {
    let parsed_args_str = vec!["logos-cli", "report", "month"];
    let parsed = logos_cli::parse_args(parsed_args_str).unwrap();
    assert!(parsed.execute().is_err());
}
