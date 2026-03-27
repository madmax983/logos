//! Monte Carlo Net Worth Projector.
//!
//! A simulator to project a range of possible future net worth outcomes
//! over a sequence of months using randomized market returns on upcoming
//! RSU vests and base net worth.

use crate::experimental::monte_carlo::{MonteCarloProjector, MonteCarloResult};
use crate::planning::fire::UpcomingVest;

/// A projector to simulate many possible future paths for net worth,
/// combining deterministic savings with stochastic RSU vest values.
#[derive(Debug, Clone)]
pub struct MonteCarloNetWorthProjector {
    initial_cents: i64,
    monthly_contribution_cents: i64,
    annual_mean_return: f64,
    annual_volatility: f64,
    seed: u64,
    upcoming_vests: Vec<UpcomingVest>,
}

impl MonteCarloNetWorthProjector {
    /// Creates a new `MonteCarloNetWorthProjector`.
    #[must_use]
    pub const fn new(
        initial_cents: i64,
        monthly_contribution_cents: i64,
        annual_mean_return: f64,
        annual_volatility: f64,
        seed: u64,
    ) -> Self {
        Self {
            initial_cents,
            monthly_contribution_cents,
            annual_mean_return,
            annual_volatility,
            seed,
            upcoming_vests: Vec::new(),
        }
    }

    /// Registers an `UpcomingVest` to be included in the stochastic projection.
    pub fn add_upcoming_vest(&mut self, vest: UpcomingVest) {
        self.upcoming_vests.push(vest);
    }

    /// Runs the Monte Carlo simulation for a given number of months and paths.
    #[must_use]
    pub fn run(&self, months: u16, paths: u32) -> MonteCarloResult {
        // Here we build a simpler version that treats vests as single lump sums
        // occurring at a specific month and applying market growth to them from that point on.

        if paths == 0 {
            return MonteCarloResult {
                p5_cents: self.initial_cents,
                median_cents: self.initial_cents,
                p95_cents: self.initial_cents,
            };
        }

        let mut final_outcomes: Vec<i64> = Vec::with_capacity(paths as usize);

        for path_idx in 0..paths {
            // We use the existing MonteCarloProjector to run a path for the base amount
            let base_mc = MonteCarloProjector::new(
                self.initial_cents,
                self.monthly_contribution_cents,
                self.annual_mean_return,
                self.annual_volatility,
                self.seed.wrapping_add(path_idx.into()), // Vary seed per path
            );

            // To simulate the full path, we actually just run 1 path from the base MC
            let base_result = base_mc.run(months, 1);
            let mut path_final_cents = base_result.median_cents;

            // For each vest, calculate how many months it has to grow *after* it vests
            for vest in &self.upcoming_vests {
                let vest_month = vest.days_to_vest.div_ceil(30); // approx months

                if vest_month <= months {
                    let months_to_grow = months - vest_month;

                    // Vest value at vest time (no haircut because we are explicitly modeling the volatility!)
                    let vest_value_at_vest =
                        vest.avg_close_price_cents.saturating_mul(vest.units.into());

                    let vest_mc = MonteCarloProjector::new(
                        vest_value_at_vest,
                        0, // no monthly contribution on this specific vest bucket
                        self.annual_mean_return,
                        self.annual_volatility,
                        self.seed
                            .wrapping_add(path_idx.into())
                            .wrapping_add(vest.days_to_vest.into()),
                    );

                    let vest_result = vest_mc.run(months_to_grow, 1);
                    path_final_cents = path_final_cents.saturating_add(vest_result.median_cents);
                }
            }

            final_outcomes.push(path_final_cents);
        }

        final_outcomes.sort_unstable();

        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let lower_bound_idx = (f64::from(paths) * 0.05).floor() as usize;

        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let median_idx = (f64::from(paths) * 0.50).floor() as usize;

        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let upper_bound_idx = (f64::from(paths) * 0.95).floor() as usize;

        let safe_lower_bound_idx = lower_bound_idx.clamp(0, paths.saturating_sub(1) as usize);
        let safe_median_idx = median_idx.clamp(0, paths.saturating_sub(1) as usize);
        let safe_upper_bound_idx = upper_bound_idx.clamp(0, paths.saturating_sub(1) as usize);

        MonteCarloResult {
            p5_cents: final_outcomes[safe_lower_bound_idx],
            median_cents: final_outcomes[safe_median_idx],
            p95_cents: final_outcomes[safe_upper_bound_idx],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monte_carlo_net_worth_projector() {
        let mut projector = MonteCarloNetWorthProjector::new(
            10_000_000, // $100k initial
            100_000,    // $1k monthly
            0.07,       // 7% return
            0.15,       // 15% volatility
            42,         // seed
        );

        projector.add_upcoming_vest(UpcomingVest {
            avg_close_price_cents: 10_000, // $100 price
            units: 500,                    // $50k vest
            days_to_vest: 60,              // 2 months
        });

        // 12 months, 100 paths
        let result = projector.run(12, 100);

        // median should be roughly $100k + $12k savings + $50k vest = $162k + growth
        assert!(result.median_cents > 162_000_000);
        assert!(result.p5_cents < result.median_cents);
        assert!(result.median_cents < result.p95_cents);
    }
}
