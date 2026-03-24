//! FIRE Goal Seeker Module.
//!
//! Answers the question: "How much do I need to save each month to reach my FIRE goal?"
//!
//! Connects the `FireSimulator`'s target generation with the `MonteCarloProjector`'s
//! probabilistic outcomes. It uses binary search to find the exact monthly contribution
//! required to hit your FIRE number in a specific number of months, at a specified confidence level.

use crate::experimental::monte_carlo::MonteCarloProjector;
use crate::planning::fire::FireSimulator;

/// The target confidence level for the goal seeker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfidenceLevel {
    /// 5th percentile (Conservative: 95% chance of reaching the goal).
    Conservative,
    /// 50th percentile (Median: 50% chance of reaching the goal).
    Moderate,
    /// 95th percentile (Aggressive: 5% chance of reaching the goal).
    Aggressive,
}

/// A simulator to find the required monthly savings for FIRE.
#[derive(Debug, Clone)]
pub struct FireGoalSeeker {
    fire_sim: FireSimulator,
    initial_cents: i64,
    annual_mean_return: f64,
    annual_volatility: f64,
    seed: u64,
}

impl FireGoalSeeker {
    /// Creates a new `FireGoalSeeker`.
    #[must_use]
    pub const fn new(
        fire_sim: FireSimulator,
        initial_cents: i64,
        annual_mean_return: f64,
        annual_volatility: f64,
        seed: u64,
    ) -> Self {
        Self {
            fire_sim,
            initial_cents,
            annual_mean_return,
            annual_volatility,
            seed,
        }
    }

    /// Finds the required monthly contribution to reach the FIRE number in `months`
    /// at the given `confidence_level`. Returns the required cents per month.
    ///
    /// Performs a binary search over possible monthly contributions.
    #[must_use]
    pub fn seek_monthly_contribution(
        &self,
        months: u16,
        confidence: ConfidenceLevel,
    ) -> Option<i64> {
        let target_cents = self.fire_sim.fire_number_cents();
        if target_cents == 0 {
            return Some(0);
        }
        if target_cents == i64::MAX {
            return None;
        }

        if months == 0 {
            if self.initial_cents >= target_cents {
                return Some(0);
            }
            return None;
        }

        let mut low = 0_i64;
        let mut high = target_cents;
        let mut best = None;
        let paths = 1000; // Fast simulation for search

        for _ in 0..64 {
            if low > high {
                break;
            }
            let mid = low + (high - low) / 2;

            let projector = MonteCarloProjector::new(
                self.initial_cents,
                mid,
                self.annual_mean_return,
                self.annual_volatility,
                self.seed,
            );

            let result = projector.run(months, paths);
            let outcome = match confidence {
                ConfidenceLevel::Conservative => result.p5_cents,
                ConfidenceLevel::Moderate => result.median_cents,
                ConfidenceLevel::Aggressive => result.p95_cents,
            };

            if outcome >= target_cents {
                best = Some(mid);
                high = mid - 1; // Try to find a smaller sufficient contribution
            } else {
                low = mid + 1;
            }
        }

        best
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::planning::fire::{FireConfig, FireSimulator};

    #[test]
    fn test_goal_seeker() {
        let mut fire_sim = FireSimulator::new(500_000); // $5k expenses
        fire_sim.set_config(FireConfig {
            safe_withdrawal_rate_pct: 4,
        });
        // Target = $1.5M (150_000_000 cents)

        let seeker = FireGoalSeeker::new(
            fire_sim, 50_000_000, // $500k initial
            0.07,       // 7% return
            0.15,       // 15% volatility
            42,         // seed
        );

        // Find required savings to hit $1.5M in 10 years (120 months) conservatively
        let required = seeker.seek_monthly_contribution(120, ConfidenceLevel::Conservative);

        assert!(required.is_some());
        let amount = required.unwrap();

        // It should take some positive amount
        assert!(amount > 0);
        assert!(amount < 150_000_000);

        // Let's verify that using this amount actually hits the target
        let projector = MonteCarloProjector::new(50_000_000, amount, 0.07, 0.15, 42);
        let result = projector.run(120, 1000);
        assert!(result.p5_cents >= 150_000_000);
    }
}
