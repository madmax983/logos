use crate::{args::CliError, runtime::CliRuntime};
use logos_reporting::{RsuBudgetPlan, ScenarioKey};

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
        month_key.map_or_else(CliRuntime::current_month_key_local, str::to_owned);
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

/// Handles `ledger budget rsu-plan`.
///
/// # Errors
///
/// Returns an error when runtime initialization or planning fails.
#[allow(clippy::too_many_arguments)]
pub fn rsu_plan(
    month_key: Option<&str>,
    quarterly_units: u32,
    days_to_vest: u16,
    bear_price_cents: i64,
    base_price_cents: i64,
    bull_price_cents: i64,
    fixed_commitments_cents: i64,
    reserve_sweep_pct: u8,
    investing_sweep_pct: u8,
) -> Result<(), CliError> {
    let runtime = CliRuntime::new().map_err(|err| CliError::CommandRuntimeFailed {
        command: "budget.rsu-plan".to_owned(),
        message: format!("runtime initialization failed: {err}"),
    })?;
    let resolved_month_key =
        month_key.map_or_else(CliRuntime::current_month_key_local, str::to_owned);
    let plan = runtime
        .plan_rsu_budget_for_month(
            &resolved_month_key,
            quarterly_units,
            days_to_vest,
            bear_price_cents,
            base_price_cents,
            bull_price_cents,
            fixed_commitments_cents,
            reserve_sweep_pct,
            investing_sweep_pct,
        )
        .map_err(|err| CliError::CommandRuntimeFailed {
            command: "budget.rsu-plan".to_owned(),
            message: err.to_string(),
        })?;

    println!("{}", render_rsu_plan_output(&plan));
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

fn render_rsu_plan_output(plan: &RsuBudgetPlan) -> String {
    let mut lines = vec![format!(
        "budget.rsu-plan month={} conservative_budget_cents={} fixed_commitments_cents={} baseline_remaining_cents={} reserve_sweep_pct={} investing_sweep_pct={}",
        plan.month_key(),
        plan.conservative_budget_cents(),
        plan.fixed_commitments_cents(),
        plan.baseline_remaining_cents(),
        plan.reserve_sweep_pct(),
        plan.investing_sweep_pct(),
    )];
    for key in [ScenarioKey::Bear, ScenarioKey::Base, ScenarioKey::Bull] {
        if let Some(scenario) = plan.scenario(key) {
            lines.push(format!(
                "budget.rsu-scenario scenario={} monthly_income_cents={} surplus_cents={} reserve_sweep_cents={} investing_sweep_cents={} available_after_sweeps_cents={}",
                scenario_name(key),
                scenario.monthly_income_cents(),
                scenario.surplus_cents(),
                scenario.reserve_sweep_cents(),
                scenario.investing_sweep_cents(),
                scenario.available_after_sweeps_cents()
            ));
        }
    }
    lines.join("\n")
}

const fn scenario_name(key: ScenarioKey) -> &'static str {
    match key {
        ScenarioKey::Bear => "bear",
        ScenarioKey::Base => "base",
        ScenarioKey::Bull => "bull",
    }
}

#[cfg(test)]
mod tests {
    use super::{BudgetRuntime, render_budget_set_output, render_rsu_plan_output};
    use logos_reporting::{RsuBudgetPlanInput, ScenarioPriceInputs, project_rsu_budget_plan};

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

    #[test]
    fn render_rsu_plan_output_is_deterministic() {
        let input = RsuBudgetPlanInput::new(
            300,
            45,
            ScenarioPriceInputs::new(10_000, 12_000, 16_000).expect("price inputs"),
            250_000,
            60,
            30,
        )
        .expect("input");
        let plan = project_rsu_budget_plan("2026-03", &input).expect("plan");

        let output = render_rsu_plan_output(&plan);

        assert!(output.contains("budget.rsu-plan month=2026-03"));
        assert!(output.contains("budget.rsu-scenario scenario=bear"));
        assert!(output.contains("budget.rsu-scenario scenario=base"));
        assert!(output.contains("budget.rsu-scenario scenario=bull"));
    }
}
