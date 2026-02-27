use crate::{args::CliError, runtime::CliRuntime};

trait BudgetRuntime {
    fn budget_variance_for_month(
        &self,
        month_key: &str,
        budget_cents: i64,
        expense_account_prefix: &str,
    ) -> i64;
}

impl BudgetRuntime for CliRuntime {
    fn budget_variance_for_month(
        &self,
        month_key: &str,
        budget_cents: i64,
        expense_account_prefix: &str,
    ) -> i64 {
        Self::budget_variance_for_month(self, month_key, budget_cents, expense_account_prefix)
    }
}

/// Handles `ledger budget set`.
///
/// # Errors
///
/// Returns an error when runtime initialization fails.
pub fn set(
    month_key: Option<&str>,
    budget_cents: i64,
    expense_account_prefix: &str,
) -> Result<(), CliError> {
    let mut runtime = CliRuntime::new().map_err(|err| CliError::CommandRuntimeFailed {
        command: "budget.set".to_owned(),
        message: format!("runtime initialization failed: {err}"),
    })?;
    let resolved_month_key =
        month_key.map_or_else(CliRuntime::current_month_key_utc, str::to_owned);
    runtime
        .set_budget_target_for_month(&resolved_month_key, expense_account_prefix, budget_cents)
        .map_err(|err| CliError::CommandRuntimeFailed {
            command: "budget.set".to_owned(),
            message: err.to_string(),
        })?;
    let output = render_budget_set_output(
        &runtime,
        &resolved_month_key,
        budget_cents,
        expense_account_prefix,
    );
    println!("{output}");
    Ok(())
}

fn render_budget_set_output(
    runtime: &impl BudgetRuntime,
    month_key: &str,
    budget_cents: i64,
    expense_account_prefix: &str,
) -> String {
    let variance_cents =
        runtime.budget_variance_for_month(month_key, budget_cents, expense_account_prefix);
    format!(
        "budget.set month={month_key} budget_cents={budget_cents} actual_prefix={expense_account_prefix} variance_cents={variance_cents}"
    )
}

#[cfg(test)]
mod tests {
    use super::{BudgetRuntime, render_budget_set_output};

    struct FakeBudgetRuntime {
        variance_cents: i64,
    }

    impl BudgetRuntime for FakeBudgetRuntime {
        fn budget_variance_for_month(
            &self,
            _month_key: &str,
            _budget_cents: i64,
            _expense_account_prefix: &str,
        ) -> i64 {
            self.variance_cents
        }
    }

    #[test]
    fn render_budget_set_output_is_deterministic() {
        let runtime = FakeBudgetRuntime {
            variance_cents: -1_250,
        };
        let output = render_budget_set_output(&runtime, "2026-03", 5_000, "expenses:");

        assert_eq!(
            output,
            "budget.set month=2026-03 budget_cents=5000 actual_prefix=expenses: variance_cents=-1250"
        );
    }
}
