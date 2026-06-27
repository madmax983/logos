#[test]
fn execute_commands_propagate_errors_without_db() {
    let exe = env!("CARGO_BIN_EXE_logos-cli");

    let commands = vec![
        vec!["db", "migrate"],
        vec![
            "txn",
            "add",
            "--description",
            "test",
            "--debit-account",
            "assets",
            "--credit-account",
            "expenses",
            "--amount-cents",
            "100",
        ],
        vec!["analytics", "sankey"],
        vec!["import", "pdf", "--file", "dummy.pdf"],
        vec!["fetch", "list-runs"],
        vec!["reconcile", "list"],
        vec!["month", "autopilot", "--confirm-close"],
        vec!["close", "month", "--run-id", "test"],
        vec!["budget", "set", "--budget-cents", "100"],
        vec!["report", "month"],
    ];

    for args in commands {
        let output = std::process::Command::new(exe)
            .args(&args)
            .env_remove("DATABASE_URL")
            .output()
            .unwrap_or_else(|_| panic!("failed to execute command: {args:?}"));

        assert!(
            !output.status.success(),
            "Command {args:?} unexpectedly succeeded"
        );
    }
}
