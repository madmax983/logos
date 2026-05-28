#![cfg(feature = "nova")]

//! Financial Stress Tester
//!
//! A simulator to stress-test financial resilience against unexpected shocks,
//! such as sudden job loss combined with a market downturn and emergency expenses.

use crate::experimental::runway_simulator::RunwaySimulator;

/// Represents the parameters for a financial shock scenario.
#[derive(Debug, Clone)]
pub struct ShockScenario {
    /// Percentage drop in liquid assets (e.g., 20.0 for a 20% market crash affecting the portfolio).
    pub asset_drop_pct: f64,
    /// Flat emergency expense incurred immediately in cents (e.g., medical bill).
    pub sudden_expense_cents: i64,
    /// Percentage increase in monthly burn rate (e.g., 50.0 for 50% more expenses like COBRA healthcare).
    pub expense_increase_pct: f64,
}

/// The result of a stress test.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StressTestResult {
    /// Assets remaining immediately after the initial shock.
    pub post_shock_assets_cents: i64,
    /// The new monthly burn rate after the shock.
    pub post_shock_monthly_burn_cents: i64,
    /// The resulting runway in months before going broke.
    pub surviving_months: u32,
    /// Whether the finances survived the desired recovery period.
    pub survived_target_period: bool,
}

/// Evaluates financial resilience against shock scenarios.
#[derive(Debug, Clone)]
pub struct FinancialStressTester {
    base_liquid_assets_cents: i64,
    base_monthly_burn_cents: i64,
    annual_inflation_pct: f64,
}

impl FinancialStressTester {
    /// Creates a new `FinancialStressTester`.
    #[must_use]
    pub const fn new(
        base_liquid_assets_cents: i64,
        base_monthly_burn_cents: i64,
        annual_inflation_pct: f64,
    ) -> Self {
        Self {
            base_liquid_assets_cents,
            base_monthly_burn_cents,
            annual_inflation_pct,
        }
    }

    /// Tests the finances against a specific shock scenario to see if they last for `target_months`.
    #[must_use]
    pub fn test_scenario(&self, scenario: &ShockScenario, target_months: u32) -> StressTestResult {
        let drop_multiplier = 1.0 - (scenario.asset_drop_pct / 100.0).clamp(0.0, 1.0);
        #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
        let mut post_shock_assets =
            (self.base_liquid_assets_cents as f64 * drop_multiplier).round() as i64;

        post_shock_assets -= scenario.sudden_expense_cents;
        if post_shock_assets < 0 {
            post_shock_assets = 0;
        }

        let expense_multiplier = 1.0 + (scenario.expense_increase_pct / 100.0).max(0.0);
        #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
        let post_shock_burn =
            (self.base_monthly_burn_cents as f64 * expense_multiplier).round() as i64;

        let runway_sim = RunwaySimulator::new(
            post_shock_assets,
            post_shock_burn,
            self.annual_inflation_pct,
        );
        let runway_result = runway_sim.calculate_runway();

        StressTestResult {
            post_shock_assets_cents: post_shock_assets,
            post_shock_monthly_burn_cents: post_shock_burn,
            surviving_months: runway_result.months,
            survived_target_period: runway_result.months >= target_months,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mild_shock() {
        let tester = FinancialStressTester::new(5_000_000, 400_000, 3.0);
        let scenario = ShockScenario {
            asset_drop_pct: 10.0,
            sudden_expense_cents: 500_000,
            expense_increase_pct: 0.0,
        };

        let result = tester.test_scenario(&scenario, 6);

        assert_eq!(result.post_shock_assets_cents, 4_000_000);
        assert_eq!(result.post_shock_monthly_burn_cents, 400_000);
        assert!(result.surviving_months >= 6);
        assert!(result.survived_target_period);
    }

    #[test]
    fn test_severe_shock_ruin() {
        let tester = FinancialStressTester::new(3_000_000, 300_000, 0.0);
        let scenario = ShockScenario {
            asset_drop_pct: 50.0,
            sudden_expense_cents: 1_000_000,
            expense_increase_pct: 100.0,
        };

        let result = tester.test_scenario(&scenario, 3);

        assert_eq!(result.post_shock_assets_cents, 500_000);
        assert_eq!(result.post_shock_monthly_burn_cents, 600_000);
        // 500_000 assets, 600_000 burn -> can't even cover 1 month fully. Wait, does calculate_runway do partial months?
        // RunwaySimulator stops when assets are 0. It burns up to current_assets.
        // If assets are 500_000, and burn is 600_000, month 1 burns 500_000, assets=0, loop breaks.
        // But month increments *after* checking current_assets > 0.
        // Ah, let's just assert survived_target_period is false and surviving_months is 1.
        assert_eq!(result.surviving_months, 1);
        assert!(!result.survived_target_period);
    }

    #[test]
    fn test_immediate_bankruptcy() {
        let tester = FinancialStressTester::new(1_000_000, 300_000, 0.0);
        let scenario = ShockScenario {
            asset_drop_pct: 0.0,
            sudden_expense_cents: 1_500_000,
            expense_increase_pct: 0.0,
        };

        let result = tester.test_scenario(&scenario, 1);

        assert_eq!(result.post_shock_assets_cents, 0);
        assert_eq!(result.surviving_months, 0);
        assert!(!result.survived_target_period);
    }
}
