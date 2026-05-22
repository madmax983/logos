//! Coast FIRE Simulator
//!
//! A simulator that calculates when you can stop saving. It takes your FIRE target
//! and reverse-engineers the "Coast FIRE" number: the amount of money you need invested
//! today so that, assuming a specific real growth rate, it will grow to your FIRE number
//! by your target retirement date without any additional contributions.

use crate::planning::fire::FireSimulator;

/// Represents the result of a Coast FIRE calculation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoastFireResult {
    /// The actual target FIRE number we are aiming for.
    pub fire_target_cents: i64,
    /// The amount needed *today* to coast to the target.
    pub coast_fire_cents: i64,
    /// Whether the current safe net worth already meets or exceeds the Coast FIRE number.
    pub is_coasting: bool,
}

/// A simulator for Coast FIRE milestones.
#[derive(Debug, Clone)]
pub struct CoastFireSimulator {
    fire_sim: FireSimulator,
    annual_growth_rate_pct: f64,
    years_to_retirement: u8,
}

impl CoastFireSimulator {
    /// Creates a new `CoastFireSimulator`.
    ///
    /// # Arguments
    /// * `fire_sim` - The base FIRE simulator that defines your expenses and target.
    /// * `annual_growth_rate_pct` - Expected real annual return (e.g., 7.0 for 7%).
    /// * `years_to_retirement` - How many years until you plan to draw down the money.
    #[must_use]
    pub const fn new(
        fire_sim: FireSimulator,
        annual_growth_rate_pct: f64,
        years_to_retirement: u8,
    ) -> Self {
        Self {
            fire_sim,
            annual_growth_rate_pct,
            years_to_retirement,
        }
    }

    /// Calculates the Coast FIRE milestone.
    #[must_use]
    pub fn calculate(&self) -> CoastFireResult {
        let fire_target_cents = self.fire_sim.fire_number_cents();

        if fire_target_cents == 0 {
            return CoastFireResult {
                fire_target_cents: 0,
                coast_fire_cents: 0,
                is_coasting: true,
            };
        }

        if fire_target_cents == i64::MAX {
            return CoastFireResult {
                fire_target_cents,
                coast_fire_cents: i64::MAX,
                is_coasting: false,
            };
        }

        let growth_factor = 1.0 + (self.annual_growth_rate_pct / 100.0);
        let compound_factor = growth_factor.powi(i32::from(self.years_to_retirement));

        #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
        let coast_fire_cents = if compound_factor > 0.0 {
            (fire_target_cents as f64 / compound_factor).round() as i64
        } else {
            fire_target_cents
        };

        let current_safe_nw = self.fire_sim.safe_net_worth_cents();

        CoastFireResult {
            fire_target_cents,
            coast_fire_cents,
            is_coasting: current_safe_nw >= coast_fire_cents,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::planning::fire::FireSimulator;

    #[test]
    fn test_coast_fire_calculation() {
        let mut fire_sim = FireSimulator::new(400_000); // $4k/mo expenses
        fire_sim.add_assets_liabilities(30_000_000, 0); // $300k current

        // 7% growth, 20 years to retirement
        let sim = CoastFireSimulator::new(fire_sim, 7.0, 20);
        let result = sim.calculate();

        assert_eq!(result.fire_target_cents, 120_000_000); // 1.2M FIRE target

        // 1.2M / (1.07^20) ~= 310,102.77
        let expected_coast = 31_010_280; // Expected calculated value in cents
        assert!((result.coast_fire_cents - expected_coast).abs() < 1000); // Allow small f64 delta

        // 300k < 310k
        assert!(!result.is_coasting);
    }

    #[test]
    fn test_negative_compound_factor() {
        let fire_sim = FireSimulator::new(400_000); // 1.2M target
        // -100% growth factor results in 0.0 compound factor
        let sim = CoastFireSimulator::new(fire_sim, -100.0, 20);
        let result = sim.calculate();

        assert_eq!(result.fire_target_cents, 120_000_000);
        // Coast fire target should fall back to fire target
        assert_eq!(result.coast_fire_cents, 120_000_000);
        assert!(!result.is_coasting);
    }

    #[test]
    fn test_already_coasting() {
        let mut fire_sim = FireSimulator::new(400_000); // $4k/mo expenses
        fire_sim.add_assets_liabilities(40_000_000, 0); // $400k current

        let sim = CoastFireSimulator::new(fire_sim, 7.0, 20);
        let result = sim.calculate();

        assert!(result.is_coasting);
    }

    #[test]
    fn test_zero_expenses() {
        let fire_sim = FireSimulator::new(0);
        let sim = CoastFireSimulator::new(fire_sim, 7.0, 20);
        let result = sim.calculate();

        assert_eq!(result.fire_target_cents, 0);
        assert_eq!(result.coast_fire_cents, 0);
        assert!(result.is_coasting);
    }
}
