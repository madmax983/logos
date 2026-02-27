use crate::{args::CliError, runtime::CliRuntime};

trait BudgetRuntime {
    fn budget_variance_for(&self, budget_cents: i64, expense_account_prefix: &str) -> i64;
}

impl BudgetRuntime for CliRuntime {
    fn budget_variance_for(&self, budget_cents: i64, expense_account_prefix: &str) -> i64 {
        Self::budget_variance_for(self, budget_cents, expense_account_prefix)
    }
}

/// Handles `ledger budget set`.
///
/// # Errors
///
/// Returns an error when runtime initialization fails.
pub fn set(budget_cents: i64, expense_account_prefix: &str) -> Result<(), CliError> {
    let runtime = CliRuntime::new().map_err(|err| CliError::CommandRuntimeFailed {
        command: "budget.set".to_owned(),
        message: format!("runtime initialization failed: {err}"),
    })?;
    let output = render_budget_set_output(&runtime, budget_cents, expense_account_prefix);
    println!("{output}");
    Ok(())
}

fn render_budget_set_output(
    runtime: &impl BudgetRuntime,
    budget_cents: i64,
    expense_account_prefix: &str,
) -> String {
    let variance_cents = runtime.budget_variance_for(budget_cents, expense_account_prefix);
    format!(
        "budget.set budget_cents={budget_cents} actual_prefix={expense_account_prefix} variance_cents={variance_cents}"
    )
}

#[cfg(test)]
mod tests {
    use super::{BudgetRuntime, render_budget_set_output};

    struct FakeBudgetRuntime {
        variance_cents: i64,
    }

    impl BudgetRuntime for FakeBudgetRuntime {
        fn budget_variance_for(&self, _budget_cents: i64, _expense_account_prefix: &str) -> i64 {
            self.variance_cents
        }
    }

    #[test]
    fn render_budget_set_output_is_deterministic() {
        let runtime = FakeBudgetRuntime {
            variance_cents: -1_250,
        };
        let output = render_budget_set_output(&runtime, 5_000, "expenses:");

        assert_eq!(
            output,
            "budget.set budget_cents=5000 actual_prefix=expenses: variance_cents=-1250"
        );
    }
}
