#![cfg(feature = "nova")]

//! Sabbatical Simulator
//!
//! A simulator that calculates the true cost and time delay of taking a sabbatical
//! on the journey to Financial Independence. It compares a baseline scenario (keep working)
//! against a sabbatical scenario (burn cash for N months, then return to work).

use crate::planning::fire::FireSimulator;
use crate::planning::net_worth_projector::NetWorthProjector;

/// The result of a sabbatical simulation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SabbaticalResult {
    /// Months to reach FIRE target if you don't take a sabbatical.
    pub baseline_months: u16,
    /// Months to reach FIRE target if you take the sabbatical.
    pub sabbatical_months: u16,
    /// The number of additional months you have to work due to the sabbatical.
    pub delay_months: u16,
    /// The target FIRE number used for the calculation.
    pub fire_target_cents: i64,
}

/// Simulates the impact of taking a sabbatical on your FIRE journey.
#[derive(Debug, Clone)]
pub struct SabbaticalSimulator {
    fire_sim: FireSimulator,
    initial_net_worth_cents: i64,
    monthly_savings_cents: i64,
    sabbatical_duration_months: u16,
    sabbatical_monthly_burn_cents: i64,
}

impl SabbaticalSimulator {
    /// Creates a new `SabbaticalSimulator`.
    #[must_use]
    pub const fn new(
        fire_sim: FireSimulator,
        initial_net_worth_cents: i64,
        monthly_savings_cents: i64,
        sabbatical_duration_months: u16,
        sabbatical_monthly_burn_cents: i64,
    ) -> Self {
        Self {
            fire_sim,
            initial_net_worth_cents,
            monthly_savings_cents,
            sabbatical_duration_months,
            sabbatical_monthly_burn_cents,
        }
    }

    /// Calculates the impact of the sabbatical compared to the baseline.
    #[must_use]
    pub fn calculate(&self) -> SabbaticalResult {
        let fire_target_cents = self.fire_sim.fire_number_cents();

        if fire_target_cents == 0 || self.initial_net_worth_cents >= fire_target_cents {
            return SabbaticalResult {
                baseline_months: 0,
                sabbatical_months: 0,
                delay_months: 0,
                fire_target_cents,
            };
        }

        // Baseline Projection (No Sabbatical)
        let mut baseline_projector =
            NetWorthProjector::new(self.initial_net_worth_cents, self.monthly_savings_cents);
        baseline_projector.add_milestone_cents(fire_target_cents);
        let (_, baseline_milestones) = baseline_projector.project_timeline(1200); // Project up to 100 years

        let baseline_months = baseline_milestones.first().map_or(u16::MAX, |&(_, m)| m);

        // Sabbatical Projection
        let mut sabbatical_months = u16::MAX;

        // 1. Burn phase
        let post_sabbatical_nw = self.initial_net_worth_cents.saturating_sub(
            self.sabbatical_monthly_burn_cents
                .saturating_mul(i64::from(self.sabbatical_duration_months)),
        );

        if post_sabbatical_nw >= fire_target_cents {
            // Reached FIRE during sabbatical (unlikely but theoretically possible if NW is already high and target drops)
            sabbatical_months = self.sabbatical_duration_months;
        } else if self.monthly_savings_cents > 0 {
            // 2. Return to work phase
            let mut return_projector =
                NetWorthProjector::new(post_sabbatical_nw, self.monthly_savings_cents);
            return_projector.add_milestone_cents(fire_target_cents);
            let (_, return_milestones) = return_projector.project_timeline(1200);

            if let Some(&(_, extra_months)) = return_milestones.first() {
                sabbatical_months = self.sabbatical_duration_months.saturating_add(extra_months);
            }
        }

        let delay_months = if sabbatical_months != u16::MAX && baseline_months != u16::MAX {
            sabbatical_months.saturating_sub(baseline_months)
        } else {
            u16::MAX
        };

        SabbaticalResult {
            baseline_months,
            sabbatical_months,
            delay_months,
            fire_target_cents,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sabbatical_impact() {
        let fire_sim = FireSimulator::new(500_000); // 5k expenses -> $1.5M target @ 4%

        let sim = SabbaticalSimulator::new(
            fire_sim, 50_000_000, // 500k starting NW
            1_000_000,  // 10k monthly savings
            12,         // 1 year sabbatical
            600_000,    // 6k monthly burn during sabbatical
        );

        let result = sim.calculate();

        assert_eq!(result.fire_target_cents, 150_000_000);

        // Baseline: Need to save $1M at $10k/mo = 100 months
        assert_eq!(result.baseline_months, 100);

        // Sabbatical:
        // Post-sabbatical NW: 500k - (6k * 12) = 428k
        // Remaining to save: 1.5M - 428k = 1.072M
        // Months to save after return: 1.072M / 10k = 107.2 -> 108 months
        // Total months: 12 + 108 = 120
        assert_eq!(result.sabbatical_months, 120);

        // Delay: 120 - 100 = 20 months
        // We took 12 months off, but it cost us 20 months of delay because we burned cash.
        assert_eq!(result.delay_months, 20);
    }

    #[test]
    fn test_impossible_to_reach_target() {
        let fire_sim = FireSimulator::new(500_000);

        let sim = SabbaticalSimulator::new(
            fire_sim, 0, 0, // No savings
            12, 500_000,
        );

        let result = sim.calculate();
        assert_eq!(result.baseline_months, u16::MAX);
        assert_eq!(result.sabbatical_months, u16::MAX);
    }
}
