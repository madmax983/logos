use logos_core::TransactionId;

use crate::{
    args::CliError,
    runtime::{CliRuntime, RuntimeError},
};

trait TxnPoster {
    fn post_double_entry(
        &mut self,
        description: &str,
        debit_account: &str,
        credit_account: &str,
        amount_cents: i64,
    ) -> Result<TransactionId, RuntimeError>;
}

impl TxnPoster for CliRuntime {
    fn post_double_entry(
        &mut self,
        description: &str,
        debit_account: &str,
        credit_account: &str,
        amount_cents: i64,
    ) -> Result<TransactionId, RuntimeError> {
        Self::post_double_entry(
            self,
            description,
            debit_account,
            credit_account,
            amount_cents,
        )
    }
}

/// Handles `ledger txn add`.
///
/// # Errors
///
/// Returns an error when write validation or runtime persistence fails.
pub fn add(
    description: &str,
    debit_account: &str,
    credit_account: &str,
    amount_cents: i64,
) -> Result<(), CliError> {
    let mut runtime = CliRuntime::new().map_err(|err| CliError::CommandRuntimeFailed {
        command: "txn.add".to_owned(),
        message: format!("runtime initialization failed: {err}"),
    })?;
    let transaction_id = post_double_entry(
        description,
        debit_account,
        credit_account,
        amount_cents,
        &mut runtime,
    )?;
    println!("txn.add wrote {}", transaction_id.as_str());
    Ok(())
}

fn post_double_entry(
    description: &str,
    debit_account: &str,
    credit_account: &str,
    amount_cents: i64,
    runtime: &mut impl TxnPoster,
) -> Result<TransactionId, CliError> {
    if description.trim().is_empty() {
        return Err(CliError::MissingTxnDescription);
    }

    if debit_account.trim().is_empty() {
        return Err(CliError::MissingArgValue {
            flag: "--debit-account".to_owned(),
        });
    }
    if credit_account.trim().is_empty() {
        return Err(CliError::MissingArgValue {
            flag: "--credit-account".to_owned(),
        });
    }
    if amount_cents <= 0 {
        return Err(CliError::InvalidArgValue {
            flag: "--amount-cents".to_owned(),
            value: amount_cents.to_string(),
        });
    }

    runtime
        .post_double_entry(description, debit_account, credit_account, amount_cents)
        .map_err(|err| CliError::CommandRuntimeFailed {
            command: "txn.add".to_owned(),
            message: err.to_string(),
        })
}

#[cfg(test)]
mod tests {
    use super::{TxnPoster, post_double_entry};
    use crate::runtime::RuntimeError;
    use logos_core::TransactionId;
    use logos_import::ImportError;

    #[derive(Debug, Default)]
    struct FakePoster {
        calls: usize,
        description: String,
        debit_account: String,
        credit_account: String,
        amount_cents: i64,
        fail_with_runtime_error: bool,
    }

    impl TxnPoster for FakePoster {
        fn post_double_entry(
            &mut self,
            description: &str,
            debit_account: &str,
            credit_account: &str,
            amount_cents: i64,
        ) -> Result<TransactionId, RuntimeError> {
            self.calls += 1;
            self.description = description.to_owned();
            self.debit_account = debit_account.to_owned();
            self.credit_account = credit_account.to_owned();
            self.amount_cents = amount_cents;

            if self.fail_with_runtime_error {
                return Err(RuntimeError::Import(ImportError::MissingColumns {
                    expected: 5,
                    found: 2,
                }));
            }

            Ok(TransactionId::new("txn-test-1"))
        }
    }

    #[test]
    fn post_double_entry_posts_with_provided_payload() {
        let mut poster = FakePoster::default();
        let txn_id = post_double_entry(
            "paycheck",
            "assets:checking",
            "income:salary",
            100_000,
            &mut poster,
        )
        .expect("post");

        assert_eq!(txn_id.as_str(), "txn-test-1");
        assert_eq!(poster.calls, 1);
        assert_eq!(poster.description, "paycheck");
        assert_eq!(poster.debit_account, "assets:checking");
        assert_eq!(poster.credit_account, "income:salary");
        assert_eq!(poster.amount_cents, 100_000);
    }

    #[test]
    fn post_double_entry_rejects_empty_description() {
        let mut poster = FakePoster::default();
        let err = post_double_entry(
            "   ",
            "assets:checking",
            "income:salary",
            100_000,
            &mut poster,
        )
        .expect_err("empty description");

        assert_eq!(poster.calls, 0);
        assert_eq!(err.to_string(), "missing transaction description");
    }

    #[test]
    fn post_double_entry_rejects_non_positive_amount() {
        let mut poster = FakePoster::default();
        let err = post_double_entry(
            "paycheck",
            "assets:checking",
            "income:salary",
            0,
            &mut poster,
        )
        .expect_err("invalid amount");

        assert_eq!(
            err.to_string(),
            "invalid value '0' for argument '--amount-cents'"
        );
        assert_eq!(poster.calls, 0);
    }

    #[test]
    fn post_double_entry_maps_runtime_error_to_cli_error() {
        let mut poster = FakePoster {
            fail_with_runtime_error: true,
            ..FakePoster::default()
        };
        let err = post_double_entry(
            "paycheck",
            "assets:checking",
            "income:salary",
            100_000,
            &mut poster,
        )
        .expect_err("runtime error");

        assert_eq!(
            err.to_string(),
            "command 'txn.add' failed at runtime: missing columns: expected 5, found 2"
        );
    }
}
