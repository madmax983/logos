#![allow(clippy::cast_precision_loss)]
use crate::args::CliError;
use logos_core::experimental::monte_carlo::MonteCarloProjector;
use logos_reporting::{RsuBudgetPlan, ScenarioKey};
use logos_runtime::AppRuntime;

trait BudgetRuntime {
    fn budget_variance_for_month(
        &self,
        month_key: &str,
        budget_cents: i64,
        expense_account_prefix: &str,
    ) -> i64;
}

impl BudgetRuntime for AppRuntime<logos_store_pg::PostgresStore> {
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
    let mut runtime = crate::runtime::init_runtime().map_err(|err| CliError::CommandRuntimeFailed {
        command: "budget.set".to_owned(),
        message: format!("{err}"),
    })?;
    let resolved_month_key =
        month_key.map_or_else(AppRuntime::<logos_store_pg::PostgresStore>::current_month_key_local, str::to_owned);
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
    let runtime = crate::runtime::init_runtime().map_err(|err| CliError::CommandRuntimeFailed {
        command: "budget.rsu-plan".to_owned(),
        message: format!("{err}"),
    })?;
    let resolved_month_key =
        month_key.map_or_else(AppRuntime::<logos_store_pg::PostgresStore>::current_month_key_local, str::to_owned);
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

use comfy_table::{Attribute, Cell, Color};

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
    table.set_header(vec!["Month", "Budget", "Actual Prefix", "Variance"]);

    let variance_color = if variance_cents >= 0 {
        Color::Green
    } else {
        Color::Red
    };
    let variance_cell = Cell::new(format!("${:.2}", (variance_cents as f64) / 100.0))
        .fg(variance_color)
        .add_attribute(Attribute::Bold);

    table.add_row(vec![
        Cell::new(month_key.to_string()),
        Cell::new(format!("${:.2}", (budget_cents as f64) / 100.0)),
        Cell::new(expense_account_prefix.to_string()),
        variance_cell,
    ]);

    format!("{table}")
}

fn render_rsu_plan_table(plan: &RsuBudgetPlan) -> comfy_table::Table {
    let mut plan_table = comfy_table::Table::new();
    plan_table.load_preset(comfy_table::presets::UTF8_FULL);
    plan_table.set_header(vec![
        Cell::new("Month")
            .fg(Color::Cyan)
            .add_attribute(Attribute::Bold),
        Cell::new("Conservative Budget")
            .fg(Color::Cyan)
            .add_attribute(Attribute::Bold),
        Cell::new("Fixed Commitments")
            .fg(Color::Cyan)
            .add_attribute(Attribute::Bold),
        Cell::new("Baseline Remaining")
            .fg(Color::Cyan)
            .add_attribute(Attribute::Bold),
        Cell::new("Reserve Sweep %")
            .fg(Color::Cyan)
            .add_attribute(Attribute::Bold),
        Cell::new("Investing Sweep %")
            .fg(Color::Cyan)
            .add_attribute(Attribute::Bold),
    ]);
    plan_table.add_row(vec![
        Cell::new(plan.month_key()).add_attribute(Attribute::Bold),
        Cell::new(format!(
            "${:.2}",
            (plan.conservative_budget_cents() as f64) / 100.0
        )),
        Cell::new(format!(
            "${:.2}",
            (plan.fixed_commitments_cents() as f64) / 100.0
        )),
        Cell::new(format!(
            "${:.2}",
            (plan.baseline_remaining_cents() as f64) / 100.0
        ))
        .add_attribute(Attribute::Bold),
        Cell::new(plan.reserve_sweep_pct()),
        Cell::new(plan.investing_sweep_pct()),
    ]);
    plan_table
}

fn render_rsu_scenario_table(plan: &RsuBudgetPlan) -> comfy_table::Table {
    let mut scenario_table = comfy_table::Table::new();
    scenario_table.load_preset(comfy_table::presets::UTF8_FULL);
    scenario_table.set_header(vec![
        Cell::new("Scenario")
            .fg(Color::Cyan)
            .add_attribute(Attribute::Bold),
        Cell::new("Monthly Income")
            .fg(Color::Cyan)
            .add_attribute(Attribute::Bold),
        Cell::new("Surplus")
            .fg(Color::Cyan)
            .add_attribute(Attribute::Bold),
        Cell::new("Reserve Sweep")
            .fg(Color::Cyan)
            .add_attribute(Attribute::Bold),
        Cell::new("Investing Sweep")
            .fg(Color::Cyan)
            .add_attribute(Attribute::Bold),
        Cell::new("Available After Sweeps")
            .fg(Color::Cyan)
            .add_attribute(Attribute::Bold),
    ]);

    for key in [ScenarioKey::Bear, ScenarioKey::Base, ScenarioKey::Bull] {
        if let Some(scenario) = plan.scenario(key) {
            let color = match key {
                ScenarioKey::Bear => Color::Red,
                ScenarioKey::Base => Color::Yellow,
                ScenarioKey::Bull => Color::Green,
            };

            scenario_table.add_row(vec![
                Cell::new(scenario_name(key))
                    .fg(color)
                    .add_attribute(Attribute::Bold),
                Cell::new(format!(
                    "${:.2}",
                    (scenario.monthly_income_cents() as f64) / 100.0
                )),
                Cell::new(format!("${:.2}", (scenario.surplus_cents() as f64) / 100.0)),
                Cell::new(format!(
                    "${:.2}",
                    (scenario.reserve_sweep_cents() as f64) / 100.0
                )),
                Cell::new(format!(
                    "${:.2}",
                    (scenario.investing_sweep_cents() as f64) / 100.0
                )),
                Cell::new(format!(
                    "${:.2}",
                    (scenario.available_after_sweeps_cents() as f64) / 100.0
                ))
                .fg(color)
                .add_attribute(Attribute::Bold),
            ]);
        }
    }
    scenario_table
}

fn render_rsu_plan_output(plan: &RsuBudgetPlan) -> String {
    let plan_table = render_rsu_plan_table(plan);
    let scenario_table = render_rsu_scenario_table(plan);

    format!("{plan_table}\n{scenario_table}")
}

fn render_monte_carlo_output(
    result: &logos_core::experimental::monte_carlo::MonteCarloResult,
) -> String {
    let mut table = comfy_table::Table::new();
    table.load_preset(comfy_table::presets::UTF8_FULL);
    table.set_header(vec!["Percentile", "Projected Outcome"]);

    table.add_row(vec![
        Cell::new("P5 (Pessimistic)").fg(Color::Red),
        Cell::new(format!("${:.2}", (result.p5_cents as f64) / 100.0)).fg(Color::Red),
    ]);
    table.add_row(vec![
        Cell::new("Median (Expected)")
            .fg(Color::Green)
            .add_attribute(Attribute::Bold),
        Cell::new(format!("${:.2}", (result.median_cents as f64) / 100.0))
            .fg(Color::Green)
            .add_attribute(Attribute::Bold),
    ]);
    table.add_row(vec![
        Cell::new("P95 (Optimistic)").fg(Color::Blue),
        Cell::new(format!("${:.2}", (result.p95_cents as f64) / 100.0)).fg(Color::Blue),
    ]);

    format!("{table}")
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

        assert!(output.contains("Month"));
        assert!(output.contains("Budget"));
        assert!(output.contains("Actual Prefix"));
        assert!(output.contains("Variance"));
        assert!(output.contains("2026-03"));
        assert!(output.contains("$50.00"));
        assert!(output.contains("expenses:"));
        assert!(output.contains("$-12.50"));
    }

    #[test]
    fn render_budget_set_output_is_deterministic_zero_variance() {
        let runtime = FakeBudgetRuntime { variance_cents: 0 };
        let output = render_budget_set_output(&runtime, "2026-04", 5_000, "expenses:");
        assert!(output.contains("$0.00"));
    }

    #[test]
    fn render_budget_set_output_is_deterministic_positive_variance() {
        let runtime = FakeBudgetRuntime {
            variance_cents: 1_250,
        };
        let output = render_budget_set_output(&runtime, "2026-05", 5_000, "expenses:");
        assert!(output.contains("$12.50"));
    }

    #[test]
    fn render_monte_carlo_output_is_deterministic() {
        let result = logos_core::experimental::monte_carlo::MonteCarloResult {
            p5_cents: 100_000,
            median_cents: 150_000,
            p95_cents: 200_000,
        };
        let output = crate::commands::budget::render_monte_carlo_output(&result);
        assert!(output.contains("P5 (Pessimistic)"));
        assert!(output.contains("$1000.00"));
        assert!(output.contains("Median (Expected)"));
        assert!(output.contains("$1500.00"));
        assert!(output.contains("P95 (Optimistic)"));
        assert!(output.contains("$2000.00"));
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

        assert!(output.contains("2026-03"));
        assert!(output.contains("Conservative Budget"));
        assert!(output.contains("Monthly Income"));
        assert!(output.contains("bear"));
        assert!(output.contains("base"));
        assert!(output.contains("bull"));
    }
}
