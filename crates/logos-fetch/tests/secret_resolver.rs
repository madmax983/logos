use std::collections::BTreeMap;

use logos_fetch::{
    OnePasswordCliSecretResolver, OutputFormat, SecretBundle, SecretRefReader, SecretResolver,
    StatementSource,
};

#[derive(Debug, Default)]
struct StubSecretRefReader {
    values: BTreeMap<String, String>,
}

impl StubSecretRefReader {
    fn with_value(mut self, secret_ref: &str, value: &str) -> Self {
        self.values.insert(secret_ref.to_owned(), value.to_owned());
        self
    }
}

impl SecretRefReader for StubSecretRefReader {
    fn read_secret_ref(&self, secret_ref: &str) -> Result<String, logos_fetch::FetchError> {
        self.values.get(secret_ref).cloned().ok_or_else(|| {
            logos_fetch::FetchError::new(format!("missing secret ref '{secret_ref}'"))
        })
    }
}

#[test]
fn one_password_resolver_reads_username_password_and_totp_refs() {
    let source = StatementSource::new(
        "pcu:checking",
        "provident-credit-union",
        "assets:checking",
        vec![OutputFormat::Pdf],
    )
    .expect("source")
    .with_secret_refs(
        "op://logos/provident/username",
        "op://logos/provident/password",
        Some("op://logos/provident/totp"),
    )
    .expect("secret refs");
    let resolver = OnePasswordCliSecretResolver::with_reader(
        StubSecretRefReader::default()
            .with_value("op://logos/provident/username", "  markm \n")
            .with_value("op://logos/provident/password", "s3cr3t\n")
            .with_value("op://logos/provident/totp", "123456\n"),
    );

    let bundle = resolver.resolve(&source).expect("bundle");

    assert_eq!(
        bundle,
        SecretBundle::new("markm", "s3cr3t", Some("123456")).expect("expected bundle")
    );
}

#[test]
fn one_password_resolver_allows_missing_optional_totp_ref() {
    let source = StatementSource::new(
        "amex:blue",
        "american-express",
        "liabilities:amex",
        vec![OutputFormat::Pdf],
    )
    .expect("source")
    .with_secret_refs("op://logos/amex/username", "op://logos/amex/password", None)
    .expect("secret refs");
    let resolver = OnePasswordCliSecretResolver::with_reader(
        StubSecretRefReader::default()
            .with_value("op://logos/amex/username", "amex-user")
            .with_value("op://logos/amex/password", "amex-pass"),
    );

    let bundle = resolver.resolve(&source).expect("bundle");

    assert_eq!(
        bundle,
        SecretBundle::new("amex-user", "amex-pass", None).expect("expected bundle")
    );
}

#[test]
fn one_password_resolver_propagates_missing_required_secret_refs() {
    let source = StatementSource::new(
        "rh:brokerage",
        "robinhood",
        "assets:brokerage",
        vec![OutputFormat::Pdf],
    )
    .expect("source")
    .with_secret_refs(
        "op://logos/robinhood/username",
        "op://logos/robinhood/password",
        None,
    )
    .expect("secret refs");
    let resolver = OnePasswordCliSecretResolver::with_reader(
        StubSecretRefReader::default().with_value("op://logos/robinhood/username", "hood-user"),
    );

    let err = resolver
        .resolve(&source)
        .expect_err("missing password ref should fail");

    assert!(
        err.to_string()
            .contains("missing secret ref 'op://logos/robinhood/password'"),
        "unexpected error: {err}"
    );
}

#[test]
fn secret_bundle_accessors() {
    use logos_fetch::SecretBundle;
    let bundle = SecretBundle::new("user", "pass", Some("123456")).expect("bundle");
    assert_eq!(bundle.username(), "user");
    assert_eq!(bundle.password(), "pass");
    assert_eq!(bundle.totp_code(), Some("123456"));
}
