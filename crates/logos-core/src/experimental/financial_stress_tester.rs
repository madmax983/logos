#![cfg(feature = "nova")]

//! Financial Stress Tester Module
//!
//! A simulator to run stress tests on a FIRE (Financial Independence, Retire Early) plan
//! against catastrophic scenarios, such as massive market drops, hyperinflation,
//! or a sudden loss of all expected unvested RSUs.
//!
//! This answers the question: "Is my plan resilient to absolute disaster, or
//! am I relying on everything going perfectly?"

use crate::planning::fire::FireSimulator;

/// Represents a specific financial catastrophe to simulate.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StressScenario {
    /// A sudden drop in the value of all liquid assets (e.g., 50% market crash).
    AssetDrop { drop_pct: f64 },
    /// A sudden spike in monthly expenses due to inflation or unexpected medical costs.
    ExpenseSpike { increase_pct: f64 },
    /// All upcoming RSUs instantly become worthless (e.g., company goes bankrupt).
    TotalRsuWipeout,
    /// A combination of multiple disasters: asset drop + expense spike + RSU wipeout.
    PerfectStorm {
        asset_drop_pct: f64,
        expense_increase_pct: f64,
    },
}

/// The result of running a financial stress test.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StressTestResult {
    /// Original FIRE number before the stress event.
    pub original_fire_number_cents: i64,
    /// Original safe net worth before the stress event.
    pub original_net_worth_cents: i64,
    /// Original progress to FIRE.
    pub original_progress_pct: u8,
    /// New FIRE number after the stress event (may increase if expenses spike).
    pub stressed_fire_number_cents: i64,
    /// New safe net worth after the stress event (may decrease if assets drop).
    pub stressed_net_worth_cents: i64,
    /// New progress to FIRE after the stress event.
    pub stressed_progress_pct: u8,
    /// Whether the FIRE goal is still met despite the disaster.
    pub still_fire: bool,
}

/// A simulator for testing financial resilience.
#[derive(Debug, Clone)]
pub struct FinancialStressTester {
    simulator: FireSimulator,
    assets_cents: i64,
    liabilities_cents: i64,
}

impl FinancialStressTester {
    /// Creates a new `FinancialStressTester` based on your current FIRE plan.
    ///
    /// Requires the underlying asset and liability figures to accurately simulate
    /// total RSU wipeouts, as the base `FireSimulator` does not expose them.
    #[must_use]
    pub const fn new(simulator: FireSimulator, assets_cents: i64, liabilities_cents: i64) -> Self {
        Self {
            simulator,
            assets_cents,
            liabilities_cents,
        }
    }

    /// Runs a specific stress scenario against the FIRE plan.
    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    pub fn run_scenario(&self, scenario: StressScenario) -> StressTestResult {
        let mut stressed_sim = self.simulator.clone();

        let original_fire_number_cents = stressed_sim.fire_number_cents();
        let original_net_worth_cents = stressed_sim.safe_net_worth_cents();
        let original_progress_pct = stressed_sim.fire_progress_pct();

        match scenario {
            StressScenario::AssetDrop { drop_pct } => {
                let current_assets = stressed_sim.safe_net_worth_cents();
                let drop_amount = (current_assets as f64 * (drop_pct / 100.0)).round() as i64;
                stressed_sim.add_assets_liabilities(0, drop_amount);
            }
            StressScenario::ExpenseSpike { increase_pct } => {
                let current_expenses = stressed_sim.monthly_expenses_cents();
                let new_expenses =
                    (current_expenses as f64 * (1.0 + increase_pct / 100.0)).round() as i64;

                let mut new_sim = FireSimulator::new(new_expenses);
                new_sim.add_assets_liabilities(stressed_sim.safe_net_worth_cents(), 0);
                stressed_sim = new_sim;
            }
            StressScenario::TotalRsuWipeout => {
                let current_expenses = stressed_sim.monthly_expenses_cents();
                let mut new_sim = FireSimulator::new(current_expenses);
                new_sim.add_assets_liabilities(self.assets_cents, self.liabilities_cents);
                stressed_sim = new_sim;
            }
            StressScenario::PerfectStorm {
                asset_drop_pct,
                expense_increase_pct,
            } => {
                let current_expenses = stressed_sim.monthly_expenses_cents();
                let new_expenses =
                    (current_expenses as f64 * (1.0 + expense_increase_pct / 100.0)).round() as i64;

                let mut new_sim = FireSimulator::new(new_expenses);
                // Wipe RSUs out first by using the base assets
                let base_nw = self.assets_cents.saturating_sub(self.liabilities_cents);
                let drop_amount = (base_nw as f64 * (asset_drop_pct / 100.0)).round() as i64;

                new_sim.add_assets_liabilities(base_nw.saturating_sub(drop_amount), 0);
                stressed_sim = new_sim;
            }
        }

        StressTestResult {
            original_fire_number_cents,
            original_net_worth_cents,
            original_progress_pct,
            stressed_fire_number_cents: stressed_sim.fire_number_cents(),
            stressed_net_worth_cents: stressed_sim.safe_net_worth_cents(),
            stressed_progress_pct: stressed_sim.fire_progress_pct(),
            still_fire: stressed_sim.fire_progress_pct() >= 100,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::planning::fire::UpcomingVest;

    #[test]
    fn test_asset_drop_scenario() {
        let mut sim = FireSimulator::new(500_000); // $1.5M FIRE number @ 4% SWR
        sim.add_assets_liabilities(100_000_000, 0); // $1M NW

        let tester = FinancialStressTester::new(sim, 100_000_000, 0);
        let result = tester.run_scenario(StressScenario::AssetDrop { drop_pct: 50.0 });

        assert_eq!(result.original_fire_number_cents, 150_000_000);
        assert_eq!(result.original_net_worth_cents, 100_000_000);
        assert_eq!(result.stressed_net_worth_cents, 50_000_000);
        assert_eq!(result.stressed_fire_number_cents, 150_000_000);
        assert!(!result.still_fire);
    }

    #[test]
    fn test_expense_spike_scenario() {
        let mut sim = FireSimulator::new(500_000);
        sim.add_assets_liabilities(150_000_000, 0); // $1.5M NW

        let tester = FinancialStressTester::new(sim, 150_000_000, 0);
        let result = tester.run_scenario(StressScenario::ExpenseSpike { increase_pct: 20.0 });

        assert_eq!(result.original_progress_pct, 100);
        assert_eq!(result.original_fire_number_cents, 150_000_000);
        assert_eq!(result.stressed_fire_number_cents, 180_000_000);
        assert_eq!(result.stressed_net_worth_cents, 150_000_000);
        assert!(!result.still_fire);
    }

    #[test]
    fn test_rsu_wipeout_scenario() {
        let mut sim = FireSimulator::new(500_000); // $1.5M FIRE number
        sim.add_assets_liabilities(100_000_000, 0); // $1M base NW

        // Add an RSU vest of $1M ($100 x 10000 units), which gets heavily haircutted
        // Let's assume it gets a 50% haircut, so $500k safe value.
        // Total original NW: $1.5M (FIRE'd)
        sim.add_upcoming_vest(UpcomingVest {
            avg_close_price_cents: 10_000,
            units: 10000,
            days_to_vest: 60, // 40% haircut = 60% retained = $600k safe
        });

        let tester = FinancialStressTester::new(sim, 100_000_000, 0);

        let result = tester.run_scenario(StressScenario::TotalRsuWipeout);

        assert_eq!(result.original_net_worth_cents, 160_000_000);
        assert_eq!(result.stressed_net_worth_cents, 100_000_000); // Back to base NW
        assert!(!result.still_fire);
    }

    #[test]
    fn test_perfect_storm_scenario() {
        let mut sim = FireSimulator::new(500_000); // $1.5M FIRE number
        sim.add_assets_liabilities(200_000_000, 0); // $2M base NW

        sim.add_upcoming_vest(UpcomingVest {
            avg_close_price_cents: 10_000,
            units: 10000,
            days_to_vest: 60, // $600k safe
        });

        let tester = FinancialStressTester::new(sim, 200_000_000, 0);

        let result = tester.run_scenario(StressScenario::PerfectStorm {
            asset_drop_pct: 50.0,
            expense_increase_pct: 50.0,
        });

        // Original NW: 2.6M
        assert_eq!(result.original_net_worth_cents, 260_000_000);

        // Stressed NW: RSUs wiped ($0), Base NW drops 50% ($1M)
        assert_eq!(result.stressed_net_worth_cents, 100_000_000);

        // Expenses spike to $7,500/mo -> FIRE number becomes $2.25M
        assert_eq!(result.stressed_fire_number_cents, 225_000_000);
        assert!(!result.still_fire);
    }
}
