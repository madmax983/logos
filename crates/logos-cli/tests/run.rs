#[test]
fn test_logos_cli_run_executes_successfully() {
    let argv = vec!["logos-cli", "--help"];
    assert!(logos_cli::run(argv).is_ok());
}

#[test]
fn test_logos_cli_run_fails_with_invalid_command() {
    let argv = vec!["logos-cli", "unknown-command"];
    assert!(logos_cli::run(argv).is_err());
}
