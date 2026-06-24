#![cfg(feature = "nova")]

//! Sabbatical Simulator
//!
//! A simulator that calculates the impact of taking a career break (sabbatical)
//! on your FIRE (Financial Independence, Retire Early) journey.
//! It compares the baseline time to FIRE versus the time to FIRE if you take
//! a specific duration off work, pausing savings and living off a separate
//! sabbatical fund.

use crate::planning::fire::FireSimulator;
use crate::planning::net_worth_projector::NetWorthProjector;

/// The result of simulating a sabbatical.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SabbaticalResult {
    /// Months to reach FIRE target without taking a sabbatical.
    pub baseline_months: u16,
    /// Months to reach FIRE target including the sabbatical duration.
    pub sabbatical_months: u16,
    /// Additional months delayed on top of the sabbatical duration.
    pub opportunity_cost_months: u16,
    /// The total months delayed due to taking the sabbatical.
    pub total_delay_months: u16,
}

/// A simulator that evaluates the cost of a sabbatical on FIRE timeline.
#[derive(Debug, Clone)]
pub struct SabbaticalSimulator {
    fire_sim: FireSimulator,
    projector: NetWorthProjector,
    sabbatical_duration_months: u16,
    sabbatical_monthly_expenses_cents: i64,
}

impl SabbaticalSimulator {
    /// Creates a new `SabbaticalSimulator`.
    ///
    /// # Arguments
    /// * `fire_sim` - The base FIRE simulator defining target and normal expenses.
    /// * `projector` - The net worth projector defining current trajectory.
    /// * `sabbatical_duration_months` - How many months the sabbatical will last.
    /// * `sabbatical_monthly_expenses_cents` - How much you will spend per month during the sabbatical.
    #[must_use]
    pub const fn new(
        fire_sim: FireSimulator,
        projector: NetWorthProjector,
        sabbatical_duration_months: u16,
        sabbatical_monthly_expenses_cents: i64,
    ) -> Self {
        Self {
            fire_sim,
            projector,
            sabbatical_duration_months,
            sabbatical_monthly_expenses_cents,
        }
    }

    /// Calculates the impact of the sabbatical on reaching the FIRE number.
    #[must_use]
    pub fn simulate(&self) -> SabbaticalResult {
        let fire_target = self.fire_sim.fire_number_cents();

        // 1. Calculate Baseline timeline
        let mut baseline_projector = self.projector.clone();
        baseline_projector.add_milestone_cents(fire_target);
        let (_, baseline_milestones) = baseline_projector.project_timeline(1200);
        let baseline_months = baseline_milestones
            .iter()
            .find(|(cents, _)| *cents == fire_target)
            .map_or(1200, |&(_, month)| month);

        // 2. Calculate Sabbatical timeline
        // The idea is we "burn" some net worth for the duration of the sabbatical,
        // and we pause our monthly savings. Then we resume the projector.

        let total_sabbatical_cost = self.sabbatical_monthly_expenses_cents.saturating_mul(i64::from(self.sabbatical_duration_months));

        let original_initial_net_worth = self.projector.initial_net_worth_cents();
        let new_initial_net_worth = original_initial_net_worth.saturating_sub(total_sabbatical_cost);

        let mut adjusted_projector = self.projector.clone();
        adjusted_projector.set_initial_net_worth_cents(new_initial_net_worth);
        adjusted_projector.add_milestone_cents(fire_target);

        let (_, sabbatical_milestones) = adjusted_projector.project_timeline(1200);
        let post_sabbatical_months = sabbatical_milestones
            .iter()
            .find(|(cents, _)| *cents == fire_target)
            .map_or(1200, |&(_, month)| month);

        let total_sabbatical_timeline = post_sabbatical_months.saturating_add(self.sabbatical_duration_months);

        let total_delay_months = total_sabbatical_timeline.saturating_sub(baseline_months);
        let opportunity_cost_months = total_delay_months.saturating_sub(self.sabbatical_duration_months);

        SabbaticalResult {
            baseline_months,
            sabbatical_months: total_sabbatical_timeline,
            opportunity_cost_months,
            total_delay_months,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sabbatical_impact() {
        let fire_sim = FireSimulator::new(500_000); // Target: $1,500,000
        let projector = NetWorthProjector::new(50_000_000, 500_000); // 500k NW, 5k monthly savings

        // Takes ~200 months normally (1.5m - 500k) / 5k = 200

        let sim = SabbaticalSimulator::new(fire_sim, projector, 12, 400_000); // 1 year off, $4k/mo
        let result = sim.simulate();

        assert_eq!(result.baseline_months, 200);

        // Sabbatical cost: 12 * 4k = 48k. New NW: 452k. Target: 1.5m. Distance: 1.048m.
        // Months to cover distance: 1.048m / 5k = ~209.6 (210) months.
        // Total timeline: 12 + 210 = 222 months.
        assert_eq!(result.sabbatical_months, 222);
        assert_eq!(result.total_delay_months, 22);
        assert_eq!(result.opportunity_cost_months, 10);
    }
}
