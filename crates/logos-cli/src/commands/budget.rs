use crate::{args::CliError, runtime::CliRuntime};
use logos_core::experimental::monte_carlo::MonteCarloProjector;
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

/// Handles `ledger budget monte-carlo`.
///
/// # Errors
///
/// Returns an error when execution fails.
pub fn monte_carlo(
    initial_cents: i64,
    monthly_contribution_cents: i64,
    annual_mean_return: f64,
    annual_volatility: f64,
    seed: u64,
    months: u16,
    paths: u32,
) -> Result<(), CliError> {
    let projector = MonteCarloProjector::new(
        initial_cents,
        monthly_contribution_cents,
        annual_mean_return,
        annual_volatility,
        seed,
    );
    let result = projector.run(months, paths);
    let output = render_monte_carlo_output(&result);
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

    let mut table = comfy_table::Table::new();
    table.load_preset(comfy_table::presets::UTF8_FULL);
    table.set_header(vec![
        "Month",
        "Budget Cents",
        "Actual Prefix",
        "Variance Cents",
    ]);
    table.add_row(vec![
        month_key.to_string(),
        budget_cents.to_string(),
        expense_account_prefix.to_string(),
        variance_cents.to_string(),
    ]);

    format!("budget.set\n{table}")
}

fn render_rsu_plan_output(plan: &RsuBudgetPlan) -> String {
    let mut plan_table = comfy_table::Table::new();
    plan_table.load_preset(comfy_table::presets::UTF8_FULL);
    plan_table.set_header(vec![
        "Month",
        "Conservative Budget",
        "Fixed Commitments",
        "Baseline Remaining",
        "Reserve Sweep %",
        "Investing Sweep %",
    ]);
    plan_table.add_row(vec![
        plan.month_key().to_string(),
        plan.conservative_budget_cents().to_string(),
        plan.fixed_commitments_cents().to_string(),
        plan.baseline_remaining_cents().to_string(),
        plan.reserve_sweep_pct().to_string(),
        plan.investing_sweep_pct().to_string(),
    ]);

    let mut scenario_table = comfy_table::Table::new();
    scenario_table.load_preset(comfy_table::presets::UTF8_FULL);
    scenario_table.set_header(vec![
        "Scenario",
        "Monthly Income",
        "Surplus",
        "Reserve Sweep",
        "Investing Sweep",
        "Available After Sweeps",
    ]);

    for key in [ScenarioKey::Bear, ScenarioKey::Base, ScenarioKey::Bull] {
        if let Some(scenario) = plan.scenario(key) {
            scenario_table.add_row(vec![
                scenario_name(key).to_string(),
                scenario.monthly_income_cents().to_string(),
                scenario.surplus_cents().to_string(),
                scenario.reserve_sweep_cents().to_string(),
                scenario.investing_sweep_cents().to_string(),
                scenario.available_after_sweeps_cents().to_string(),
            ]);
        }
    }

    format!("budget.rsu-plan\n{plan_table}\n{scenario_table}")
}

fn render_monte_carlo_output(result: &logos_core::experimental::monte_carlo::MonteCarloResult) -> String {
    let mut table = comfy_table::Table::new();
    table.load_preset(comfy_table::presets::UTF8_FULL);
    table.set_header(vec!["Percentile", "Projected Outcome"]);

    table.add_row(vec![
        "P5 (Pessimistic)",
        &format!("${:.2}", (result.p5_cents as f64) / 100.0),
    ]);
    table.add_row(vec![
        "Median (Expected)",
        &format!("${:.2}", (result.median_cents as f64) / 100.0),
    ]);
    table.add_row(vec![
        "P95 (Optimistic)",
        &format!("${:.2}", (result.p95_cents as f64) / 100.0),
    ]);

    format!("budget.monte-carlo\n{table}")
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
            "budget.set\n\
            ┌─────────┬──────────────┬───────────────┬────────────────┐\n\
            │ Month   ┆ Budget Cents ┆ Actual Prefix ┆ Variance Cents │\n\
            ╞═════════╪══════════════╪═══════════════╪════════════════╡\n\
            │ 2026-03 ┆ 5000         ┆ expenses:     ┆ -1250          │\n\
            └─────────┴──────────────┴───────────────┴────────────────┘"
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

        assert!(output.contains("budget.rsu-plan"));
        assert!(output.contains("2026-03"));
        assert!(output.contains("Conservative Budget"));
        assert!(output.contains("Monthly Income"));
        assert!(output.contains("bear"));
        assert!(output.contains("base"));
        assert!(output.contains("bull"));
    }
}
