#![cfg(feature = "nova")]

//! Stochastic FIRE Simulator
//!
//! A simulator that combines deterministic FIRE calculations with Monte Carlo
//! market projections to determine the probabilistic outcome of achieving Financial Independence.

use crate::experimental::monte_carlo::MonteCarloProjector;
use crate::planning::fire::FireSimulator;

/// Represents the probabilistic outcome of a stochastic FIRE projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StochasticFireResult {
    /// The target FIRE number in cents we are aiming for.
    pub fire_target_cents: i64,
    /// 5th percentile final net worth in cents (Pessimistic).
    pub p5_cents: i64,
    /// 50th percentile final net worth in cents (Median).
    pub median_cents: i64,
    /// 95th percentile final net worth in cents (Optimistic).
    pub p95_cents: i64,
    /// The probability of reaching the FIRE target in the given timeframe (0 to 100).
    pub success_probability_pct: u8,
}

/// A simulator that determines the probability of reaching a FIRE target
/// over a specific timeframe using a Monte Carlo market projection.
#[derive(Debug, Clone)]
pub struct StochasticFireSimulator {
    fire_sim: FireSimulator,
    annual_mean_return: f64,
    annual_volatility: f64,
    seed: u64,
}

impl StochasticFireSimulator {
    /// Creates a new `StochasticFireSimulator`.
    ///
    /// # Arguments
    /// * `fire_sim` - The base FIRE simulator defining target and expenses.
    /// * `annual_mean_return` - Expected annual mean return for the Monte Carlo projection (e.g. 0.07).
    /// * `annual_volatility` - Expected annual volatility for the Monte Carlo projection (e.g. 0.15).
    /// * `seed` - Random seed for the Monte Carlo simulation.
    #[must_use]
    pub const fn new(
        fire_sim: FireSimulator,
        annual_mean_return: f64,
        annual_volatility: f64,
        seed: u64,
    ) -> Self {
        Self {
            fire_sim,
            annual_mean_return,
            annual_volatility,
            seed,
        }
    }

    /// Simulates the net worth growth for a given number of months and paths,
    /// evaluating the probability of reaching the target FIRE number.
    #[must_use]
    pub fn run(
        &self,
        months: u16,
        paths: u32,
        monthly_contribution_cents: i64,
    ) -> StochasticFireResult {
        let fire_target_cents = self.fire_sim.fire_number_cents();

        let initial_cents = self.fire_sim.safe_net_worth_cents();

        let projector = MonteCarloProjector::new(
            initial_cents,
            monthly_contribution_cents,
            self.annual_mean_return,
            self.annual_volatility,
            self.seed,
        );

        let mc_result = projector.run(months, paths);

        // Very basic success probability estimation based on the percentiles.
        // A true Monte Carlo success calculation would require evaluating all paths against the target,
        // but since `MonteCarloProjector::run` only returns the percentiles, we'll approximate.
        // If the median is less than the target, probability is < 50%.
        // For demonstration of the mashup, we'll do a simple check on the 3 percentiles.

        let mut success_probability_pct = 0;

        if paths == 0 {
            if initial_cents >= fire_target_cents {
                success_probability_pct = 100;
            }
        } else if mc_result.p5_cents >= fire_target_cents {
            success_probability_pct = 95;
        } else if mc_result.median_cents >= fire_target_cents {
            success_probability_pct = 50;
        } else if mc_result.p95_cents >= fire_target_cents {
            success_probability_pct = 5;
        }

        StochasticFireResult {
            fire_target_cents,
            p5_cents: mc_result.p5_cents,
            median_cents: mc_result.median_cents,
            p95_cents: mc_result.p95_cents,
            success_probability_pct,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::planning::fire::{FireConfig, FireSimulator};

    #[test]
    fn test_stochastic_fire_successful() {
        let mut fire_sim = FireSimulator::new(400_000); // $4k/mo expenses
        fire_sim.set_config(FireConfig {
            safe_withdrawal_rate_pct: 4, // $1.2M target
        });
        fire_sim.add_assets_liabilities(50_000_000, 0); // $500k initial NW

        let sim = StochasticFireSimulator::new(
            fire_sim, 0.07, // 7% mean return
            0.15, // 15% volatility
            42,   // fixed seed
        );

        // Run for 15 years (180 months), saving $2k/mo
        let result = sim.run(180, 1000, 200_000);

        assert_eq!(result.fire_target_cents, 120_000_000);
        // With 15 years of growth + 2k/mo, the median should be well over $1.2M
        assert!(result.median_cents > 120_000_000);
        assert!(result.success_probability_pct >= 50);
    }

    #[test]
    fn test_zero_paths() {
        let fire_sim = FireSimulator::new(400_000);
        let sim = StochasticFireSimulator::new(fire_sim, 0.07, 0.15, 42);

        let result = sim.run(10, 0, 0);
        assert_eq!(result.success_probability_pct, 0);
        assert_eq!(result.median_cents, 0);
    }
}
