use logos_core::TransactionId;

use crate::args::CliError;
use logos_runtime::{AppRuntime, RuntimeError};

trait TxnPoster {
    fn post_double_entry(
        &mut self,
        description: &str,
        debit_account: &str,
        credit_account: &str,
        amount_cents: i64,
    ) -> Result<TransactionId, RuntimeError>;

    fn apply_correction(
        &mut self,
        supersedes_id: TransactionId,
        reason: &str,
    ) -> Result<(), RuntimeError>;
}

impl TxnPoster for AppRuntime<logos_store_pg::PostgresStore> {
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

    fn apply_correction(
        &mut self,
        supersedes_id: TransactionId,
        reason: &str,
    ) -> Result<(), RuntimeError> {
        Self::apply_correction(self, supersedes_id, reason)
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
    let mut runtime =
        crate::runtime::init_runtime().map_err(|err| CliError::CommandRuntimeFailed {
            command: "txn.add".to_owned(),
            message: format!("{err}"),
        })?;
    let transaction_id = post_double_entry(
        description,
        debit_account,
        credit_account,
        amount_cents,
        &mut runtime,
    )?;
    let amount = logos_core::format::currency(amount_cents);

    let mut table = comfy_table::Table::new();
    table.load_preset(comfy_table::presets::UTF8_FULL);
    table.set_header(vec![
        comfy_table::Cell::new("Status")
            .add_attribute(comfy_table::Attribute::Bold)
            .fg(comfy_table::Color::Green),
        comfy_table::Cell::new("Transaction ID").add_attribute(comfy_table::Attribute::Bold),
        comfy_table::Cell::new("Description").add_attribute(comfy_table::Attribute::Bold),
        comfy_table::Cell::new("Amount").add_attribute(comfy_table::Attribute::Bold),
        comfy_table::Cell::new("Debit Account").add_attribute(comfy_table::Attribute::Bold),
        comfy_table::Cell::new("Credit Account").add_attribute(comfy_table::Attribute::Bold),
    ]);

    table.add_row(vec![
        comfy_table::Cell::new("✔ Added").fg(comfy_table::Color::Green),
        comfy_table::Cell::new(transaction_id.as_str()),
        comfy_table::Cell::new(description),
        comfy_table::Cell::new(amount).fg(comfy_table::Color::Blue).set_alignment(comfy_table::CellAlignment::Right),
        comfy_table::Cell::new(debit_account),
        comfy_table::Cell::new(credit_account),
    ]);

    println!("{table}");
    Ok(())
}

/// Handles `ledger txn correct`.
///
/// # Errors
///
/// Returns an error when correction validation or runtime persistence fails.
pub fn correct(supersedes_id: &str, reason: &str) -> Result<(), CliError> {
    let mut runtime =
        crate::runtime::init_runtime().map_err(|err| CliError::CommandRuntimeFailed {
            command: "txn.correct".to_owned(),
            message: format!("{err}"),
        })?;
    apply_correction(supersedes_id, reason, &mut runtime)?;
    let mut table = comfy_table::Table::new();
    table.load_preset(comfy_table::presets::UTF8_FULL);
    table.set_header(vec![
        comfy_table::Cell::new("Status")
            .add_attribute(comfy_table::Attribute::Bold)
            .fg(comfy_table::Color::Green),
        comfy_table::Cell::new("Supersedes ID").add_attribute(comfy_table::Attribute::Bold),
        comfy_table::Cell::new("Reason").add_attribute(comfy_table::Attribute::Bold),
    ]);

    table.add_row(vec![
        comfy_table::Cell::new("✔ Corrected").fg(comfy_table::Color::Green),
        comfy_table::Cell::new(supersedes_id),
        comfy_table::Cell::new(reason),
    ]);

    println!("{table}");
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

fn apply_correction(
    supersedes_id: &str,
    reason: &str,
    runtime: &mut impl TxnPoster,
) -> Result<(), CliError> {
    if supersedes_id.trim().is_empty() {
        return Err(CliError::MissingArgValue {
            flag: "--supersedes-id".to_owned(),
        });
    }
    if reason.trim().is_empty() {
        return Err(CliError::MissingArgValue {
            flag: "--reason".to_owned(),
        });
    }

    let supersedes_id =
        TransactionId::new(supersedes_id).map_err(|_| CliError::MissingArgValue {
            flag: "--supersedes-id".to_owned(),
        })?;

    runtime
        .apply_correction(supersedes_id, reason)
        .map_err(|err| CliError::CommandRuntimeFailed {
            command: "txn.correct".to_owned(),
            message: err.to_string(),
        })
}

#[cfg(test)]
mod tests {
    use super::{TxnPoster, apply_correction, post_double_entry};
    use logos_core::TransactionId;
    use logos_import::ImportError;
    use logos_runtime::RuntimeError;

    #[derive(Debug, Default)]
    struct FakePoster {
        calls: usize,
        correction_calls: usize,
        description: String,
        debit_account: String,
        credit_account: String,
        amount_cents: i64,
        correction_supersedes_id: Option<TransactionId>,
        correction_reason: String,
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

            Ok(TransactionId::new("txn-test-1").expect("valid id"))
        }

        fn apply_correction(
            &mut self,
            supersedes_id: TransactionId,
            reason: &str,
        ) -> Result<(), RuntimeError> {
            self.correction_calls += 1;
            self.correction_supersedes_id = Some(supersedes_id);
            self.correction_reason = reason.to_owned();

            if self.fail_with_runtime_error {
                return Err(RuntimeError::Import(ImportError::MissingColumns {
                    expected: 5,
                    found: 2,
                }));
            }

            Ok(())
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
        assert_eq!(err.to_string(), "Missing transaction description.");
    }

    #[test]
    fn post_double_entry_rejects_negative_amount() {
        let mut poster = FakePoster::default();
        let err = post_double_entry(
            "paycheck",
            "assets:checking",
            "income:salary",
            -1,
            &mut poster,
        )
        .expect_err("invalid amount");

        assert_eq!(
            err.to_string(),
            "Invalid value '-1' for argument '--amount-cents'."
        );
        assert_eq!(poster.calls, 0);
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
            "Invalid value '0' for argument '--amount-cents'."
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

        assert_eq!(err.to_string(), "missing columns: expected 5, found 2");
    }

    #[test]
    fn apply_correction_posts_runtime_payload() {
        let mut poster = FakePoster::default();
        apply_correction("txn-7", "fix memo", &mut poster).expect("apply correction");

        assert_eq!(poster.correction_calls, 1);
        assert_eq!(
            poster
                .correction_supersedes_id
                .as_ref()
                .map(TransactionId::as_str),
            Some("txn-7")
        );
        assert_eq!(poster.correction_reason, "fix memo");
    }

    #[test]
    fn apply_correction_rejects_empty_supersedes_id() {
        let mut poster = FakePoster::default();
        let err = apply_correction("   ", "fix memo", &mut poster).expect_err("missing id");

        assert_eq!(
            err.to_string(),
            "Missing value for argument '--supersedes-id'."
        );
        assert_eq!(poster.correction_calls, 0);
    }

    #[test]
    fn apply_correction_rejects_empty_reason() {
        let mut poster = FakePoster::default();
        let err = apply_correction("txn-7", "   ", &mut poster).expect_err("missing reason");

        assert_eq!(err.to_string(), "Missing value for argument '--reason'.");
        assert_eq!(poster.correction_calls, 0);
    }

    #[test]
    fn apply_correction_maps_runtime_error_to_cli_error() {
        let mut poster = FakePoster {
            fail_with_runtime_error: true,
            ..FakePoster::default()
        };
        let err = apply_correction("txn-9", "fix memo", &mut poster).expect_err("runtime error");

        assert_eq!(err.to_string(), "missing columns: expected 5, found 2");
    }
}
