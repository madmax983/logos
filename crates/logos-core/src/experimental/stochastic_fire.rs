//! Stochastic FIRE Simulator
//!
//! Combines Monte Carlo market projections with FIRE targets to
//! calculate the probability of reaching Financial Independence
//! within a given timeframe.

use crate::experimental::monte_carlo::{MonteCarloProjector, MonteCarloResult};
use crate::planning::fire::FireSimulator;

/// Calculates the probability of reaching a FIRE target within a timeframe.
#[derive(Debug, Clone)]
pub struct StochasticFireSimulator {
    fire_sim: FireSimulator,
    mc_projector: MonteCarloProjector,
}

impl StochasticFireSimulator {
    /// Creates a new Stochastic FIRE Simulator.
    #[must_use]
    pub const fn new(fire_sim: FireSimulator, mc_projector: MonteCarloProjector) -> Self {
        Self {
            fire_sim,
            mc_projector,
        }
    }

    /// Runs the simulation and returns the projected outcomes along with
    /// a boolean indicating if the median outcome reaches the FIRE target.
    #[must_use]
    pub fn run(&self, months: u16, paths: u32) -> (MonteCarloResult, bool) {
        let mc_result = self.mc_projector.run(months, paths);
        let fire_number = self.fire_sim.fire_number_cents();

        // If FIRE number is 0, we've already reached it.
        if fire_number == 0 {
            return (mc_result, true);
        }
        // If SWR is 0, FIRE number is i64::MAX
        if fire_number == i64::MAX {
            return (mc_result, false);
        }

        let median_success = mc_result.median_cents >= fire_number;

        (mc_result, median_success)
    }

    /// Returns a success probability estimate as a percentage (0 to 100).
    ///
    /// In this simplified implementation, it checks if the outcome percentiles
    /// reached the goal. It returns an approximate probability based on whether
    /// the 5th, 50th, or 95th percentile exceeded the FIRE target.
    #[must_use]
    pub fn evaluate_percentiles(&self, months: u16, paths: u32) -> u8 {
        let mc_result = self.mc_projector.run(months, paths);
        let fire_number = self.fire_sim.fire_number_cents();

        if fire_number == 0 {
            return 100;
        }
        if fire_number == i64::MAX {
            return 0;
        }

        if mc_result.p5_cents >= fire_number {
            95
        } else if mc_result.median_cents >= fire_number {
            50
        } else if mc_result.p95_cents >= fire_number {
            5
        } else {
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stochastic_fire_evaluation() {
        let mut fire_sim = FireSimulator::new(500_000); // 1.5M FIRE number @ 4% default
        fire_sim.add_assets_liabilities(50_000_000, 0);

        let mc = MonteCarloProjector::new(
            50_000_000, // 500k base
            500_000,    // 5k monthly
            0.07, 0.15, 42,
        );

        let stochastic_sim = StochasticFireSimulator::new(fire_sim, mc);

        // 10 years (120 months)
        let (_, median_success) = stochastic_sim.run(12, 1000);
        assert!(!median_success); // Median won't reach 1.5M in 1 year with 5k/mo

        // 30 years (360 months)
        let (_, median_success_30) = stochastic_sim.run(360, 1000);
        assert!(median_success_30); // 30 years should be enough

        let pct = stochastic_sim.evaluate_percentiles(360, 1000);
        assert!(pct >= 50);
    }

    #[test]
    fn test_zero_expenses() {
        let fire_sim = FireSimulator::new(0);
        let mc = MonteCarloProjector::new(1_000_000, 0, 0.05, 0.1, 42);
        let sim = StochasticFireSimulator::new(fire_sim, mc);

        let (_, success) = sim.run(12, 10);
        assert!(success);
        assert_eq!(sim.evaluate_percentiles(12, 10), 100);
    }
}
