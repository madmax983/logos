#![cfg(feature = "nova")]

//! Geo-Arbitrage Simulator
//!
//! Evaluates the impact of geographic arbitrage on your Financial Independence trajectory.
//! It mashes up a `FireSimulator` with cost-of-living (COL) indices and optionally adjusted
//! salaries to determine how relocating to a new city impacts your FIRE date and target.

use crate::planning::fire::FireSimulator;

#[cfg(test)]
use crate::planning::fire::FireConfig;

/// The result of a geo-arbitrage relocation simulation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelocationResult {
    /// Months to reach FIRE target in the current location.
    pub current_months: u32,
    /// Months to reach FIRE target in the new location.
    pub target_months: u32,
    /// Difference in months (negative means FIRE is achieved faster).
    pub months_saved: i32,
    /// Original FIRE target in cents.
    pub current_target_cents: i64,
    /// New FIRE target in cents based on the target location's COL.
    pub target_target_cents: i64,
}

/// Simulates relocating to a different cost-of-living area.
#[derive(Debug, Clone)]
pub struct GeoArbitrageSimulator {
    fire_sim: FireSimulator,
    initial_net_worth_cents: i64,
    current_monthly_income_cents: i64,
    current_col_index: f64,
    annual_return_pct: f64,
}

impl GeoArbitrageSimulator {
    /// Creates a new `GeoArbitrageSimulator`.
    ///
    /// # Arguments
    /// * `fire_sim` - The base FIRE simulator defining current expenses and safe withdrawal rate.
    /// * `initial_net_worth_cents` - Starting net worth.
    /// * `current_monthly_income_cents` - Current monthly income after taxes.
    /// * `current_col_index` - The cost-of-living index of the current city (e.g., 100.0 for average).
    /// * `annual_return_pct` - Expected annual return on investments (e.g., 7.0).
    #[must_use]
    pub const fn new(
        fire_sim: FireSimulator,
        initial_net_worth_cents: i64,
        current_monthly_income_cents: i64,
        current_col_index: f64,
        annual_return_pct: f64,
    ) -> Self {
        Self {
            fire_sim,
            initial_net_worth_cents,
            current_monthly_income_cents,
            current_col_index,
            annual_return_pct,
        }
    }

    /// Simulates the time to reach FIRE in a specific scenario.
    #[must_use]
    fn simulate(&self, expenses: i64, income: i64) -> (u32, i64) {
        let mut net_worth = self.initial_net_worth_cents;
        let mut months = 0;

        let monthly_return_rate = if self.annual_return_pct > 0.0 {
            (1.0 + self.annual_return_pct / 100.0).powf(1.0 / 12.0) - 1.0
        } else {
            0.0
        };

        if expenses >= income && self.initial_net_worth_cents <= 0 {
            return (u32::MAX, i64::MAX);
        }

        while months < 1200 {
            let mut current_fire_sim = FireSimulator::new(expenses);
            current_fire_sim.set_config(self.fire_sim.config());
            let target = current_fire_sim.fire_number_cents();

            if net_worth >= target {
                return (months, target);
            }

            if net_worth > 0 {
                #[allow(clippy::cast_precision_loss)]
                let returns = (net_worth as f64) * monthly_return_rate;
                #[allow(clippy::cast_possible_truncation)]
                let returns_cents = returns.round() as i64;
                net_worth += returns_cents;
            }

            let savings = income - expenses;
            net_worth += savings;

            months += 1;
        }

        (1200, i64::MAX)
    }

    /// Calculates the impact of moving to a new location.
    ///
    /// # Arguments
    /// * `target_col_index` - The COL index of the destination (e.g., 80.0 for cheaper, 150.0 for more expensive).
    /// * `target_monthly_income_cents` - The expected monthly income in the new location (to account for local pay cuts/raises).
    #[must_use]
    pub fn calculate_relocation(
        &self,
        target_col_index: f64,
        target_monthly_income_cents: i64,
    ) -> RelocationResult {
        let current_expenses = self.fire_sim.monthly_expenses_cents();

        let (current_months, current_target_cents) =
            self.simulate(current_expenses, self.current_monthly_income_cents);

        let col_multiplier = target_col_index / self.current_col_index;
        #[allow(clippy::cast_precision_loss)]
        let target_expenses_f64 = (current_expenses as f64) * col_multiplier;
        #[allow(clippy::cast_possible_truncation)]
        let target_expenses = target_expenses_f64.round() as i64;

        let (target_months, target_target_cents) =
            self.simulate(target_expenses, target_monthly_income_cents);

        #[allow(clippy::cast_possible_wrap)]
        let months_saved = if current_months == 1200 || current_months == u32::MAX {
            if target_months < 1200 {
                -(target_months as i32) // infinite to finite is a huge save, let's just use -target_months, though ideally this means "saved forever"
            } else {
                0
            }
        } else if target_months == 1200 || target_months == u32::MAX {
            i32::MAX
        } else {
            (target_months as i32) - (current_months as i32)
        };

        RelocationResult {
            current_months,
            target_months,
            months_saved,
            current_target_cents,
            target_target_cents,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_move_to_cheaper_col_same_salary() {
        let mut fire_sim = FireSimulator::new(500_000); // $5k expenses
        fire_sim.set_config(FireConfig {
            safe_withdrawal_rate_pct: 4,
        });

        let sim = GeoArbitrageSimulator::new(
            fire_sim, 0,         // $0 initial NW
            1_000_000, // $10k income
            100.0,     // Current COL: 100
            7.0,       // 7% return
        );

        // Move to a city that is 20% cheaper (COL 80), keeping the same salary.
        let result = sim.calculate_relocation(80.0, 1_000_000);

        // $5k expenses * (80/100) = $4k expenses.
        // Current target: 5000 * 12 * 25 = 1.5M.
        // Target target: 4000 * 12 * 25 = 1.2M.
        assert_eq!(result.current_target_cents, 150_000_000);
        assert_eq!(result.target_target_cents, 120_000_000);

        // Should reach FIRE significantly faster
        assert!(result.target_months < result.current_months);
        assert!(result.months_saved < 0);
    }

    #[test]
    fn test_move_to_expensive_col_with_pay_bump() {
        let fire_sim = FireSimulator::new(400_000); // $4k expenses
        let sim = GeoArbitrageSimulator::new(
            fire_sim, 10_000_000, // $100k initial NW
            800_000,    // $8k income
            100.0,      // Current COL: 100
            7.0,        // 7% return
        );

        // Move to a city that is 50% more expensive (COL 150), but get a 25% pay bump ($10k income).
        let result = sim.calculate_relocation(150.0, 1_000_000);

        // New expenses: 4000 * 1.5 = $6k.
        // Old savings: 8k - 4k = $4k.
        // New savings: 10k - 6k = $4k.
        // Same savings, but target increased from 1.2M to 1.8M.
        // Therefore, it will take longer.
        assert!(result.target_months > result.current_months);
        assert!(result.months_saved > 0);
    }
}
