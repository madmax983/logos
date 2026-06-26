//! Barista FIRE Simulator
//!
//! A simulator that calculates the "Barista FIRE" number. This is the amount of money
//! needed invested today so that, assuming a safe withdrawal rate, it covers your baseline
//! expenses *minus* the income you plan to earn from a lower-stress "barista" job.

use crate::planning::fire::FireSimulator;

/// Represents the result of a Barista FIRE calculation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BaristaFireResult {
    /// The target net worth needed to Barista FIRE.
    pub barista_fire_cents: i64,
    /// Whether the current safe net worth already meets or exceeds the Barista FIRE number.
    pub is_barista_ready: bool,
}

/// A simulator for Barista FIRE milestones.
#[derive(Debug, Clone)]
pub struct BaristaFireSimulator {
    fire_sim: FireSimulator,
    expected_barista_income_cents: i64,
}

impl BaristaFireSimulator {
    /// Creates a new `BaristaFireSimulator`.
    ///
    /// # Arguments
    /// * `fire_sim` - The base FIRE simulator that defines your total expenses and current safe net worth.
    /// * `expected_barista_income_cents` - The expected annual income from the "barista" job.
    #[must_use]
    pub const fn new(fire_sim: FireSimulator, expected_barista_income_cents: i64) -> Self {
        Self {
            fire_sim,
            expected_barista_income_cents,
        }
    }

    /// Calculates the Barista FIRE milestone.
    #[must_use]
    pub fn calculate(&self) -> BaristaFireResult {
        let swr_pct = self.fire_sim.config().safe_withdrawal_rate_pct;

        if swr_pct == 0 {
            return BaristaFireResult {
                barista_fire_cents: i64::MAX,
                is_barista_ready: false,
            };
        }

        let yearly_expenses = self.fire_sim.monthly_expenses_cents().saturating_mul(12);

        if self.expected_barista_income_cents >= yearly_expenses {
            return BaristaFireResult {
                barista_fire_cents: 0,
                is_barista_ready: true,
            };
        }

        let remaining_expenses = yearly_expenses.saturating_sub(self.expected_barista_income_cents);

        let barista_fire_cents = remaining_expenses
            .saturating_mul(100)
            .saturating_div(i64::from(swr_pct));

        let current_safe_nw = self.fire_sim.safe_net_worth_cents();

        BaristaFireResult {
            barista_fire_cents,
            is_barista_ready: current_safe_nw >= barista_fire_cents,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::planning::fire::{FireConfig, FireSimulator};

    #[test]
    fn test_barista_fire_calculation() {
        let mut fire_sim = FireSimulator::new(400_000); // $4k/mo expenses = $48k/yr
        fire_sim.add_assets_liabilities(30_000_000, 0); // $300k current

        // Standard FIRE target would be 1.2M @ 4% SWR.
        // But with a 24k/yr "barista" job, we only need to cover the remaining 24k/yr.
        // 24k / 0.04 = $600k target.
        let sim = BaristaFireSimulator::new(fire_sim, 2_400_000);
        let result = sim.calculate();

        assert_eq!(result.barista_fire_cents, 60_000_000); // 600k target
        assert!(!result.is_barista_ready); // 300k < 600k
    }

    #[test]
    fn test_already_barista_ready() {
        let mut fire_sim = FireSimulator::new(400_000); // $4k/mo expenses = $48k/yr
        fire_sim.add_assets_liabilities(70_000_000, 0); // $700k current

        let sim = BaristaFireSimulator::new(fire_sim, 2_400_000);
        let result = sim.calculate();

        assert!(result.is_barista_ready);
    }

    #[test]
    fn test_barista_income_covers_all_expenses() {
        let fire_sim = FireSimulator::new(400_000); // $4k/mo expenses = $48k/yr

        // Income exceeds expenses.
        let sim = BaristaFireSimulator::new(fire_sim, 5_000_000); // $50k/yr
        let result = sim.calculate();

        assert_eq!(result.barista_fire_cents, 0);
        assert!(result.is_barista_ready);
    }

    #[test]
    fn test_zero_swr() {
        let mut fire_sim = FireSimulator::new(400_000);
        fire_sim.set_config(FireConfig {
            safe_withdrawal_rate_pct: 0,
        });
        let sim = BaristaFireSimulator::new(fire_sim, 2_400_000);
        let result = sim.calculate();

        assert_eq!(result.barista_fire_cents, i64::MAX);
        assert!(!result.is_barista_ready);
    }
}
