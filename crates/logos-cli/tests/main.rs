use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_logos_cli_main_prints_error_and_exits() {
    let mut cmd = Command::cargo_bin("logos-cli").unwrap();
    cmd.arg("unknown-command");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Error: Unknown command 'unknown-command'."));
}
