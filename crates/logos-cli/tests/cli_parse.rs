#[test]
fn parses_txn_add_command() {
    let args = vec!["ledger", "txn", "add", "--description", "paycheck"];
    let parsed = logos_cli::parse_args(args).expect("parse");

    assert_eq!(parsed.command_path(), "txn.add");
}
