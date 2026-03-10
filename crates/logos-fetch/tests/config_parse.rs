use logos_fetch::{OutputFormat, StatementSourceConfig};

#[test]
fn parses_source_config_with_1password_references() {
    let toml = r#"
        [[sources]]
        source_id = "pcu:checking"
        institution_id = "provident-credit-union"
        ledger_account = "assets:checking"
        format_preference = ["csv", "pdf"]
        username_secret_ref = "op://logos/provident/username"
        password_secret_ref = "op://logos/provident/password"
        totp_secret_ref = "op://logos/provident/totp"
    "#;

    let config = StatementSourceConfig::from_toml(toml).expect("config parses");
    assert_eq!(
        config.sources()[0].preferred_format(),
        Some(OutputFormat::Csv)
    );
    assert_eq!(
        config.sources()[0].username_secret_ref(),
        "op://logos/provident/username"
    );
}

#[test]
fn rejects_empty_secret_references() {
    let toml = r#"
        [[sources]]
        source_id = "pcu:checking"
        institution_id = "provident-credit-union"
        ledger_account = "assets:checking"
        format_preference = ["pdf"]
        username_secret_ref = ""
        password_secret_ref = "op://logos/provident/password"
    "#;

    assert!(StatementSourceConfig::from_toml(toml).is_err());
}
