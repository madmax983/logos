//! Sabbatical Planner
//!
//! Evaluates the true cost of taking a career break. It calculates how a sabbatical
//! affects your long-term financial goals by comparing two timelines:
//! 1. The "Standard" timeline where you keep working and saving.
//! 2. The "Sabbatical" timeline where you draw down your assets for N months,
//!    then return to work.
//!
//! 🌟 Nova Mashup: We combine the `RunwaySimulator` (to ensure you don't run out of money
//! during the break) with compound growth math to calculate the exact "Goal Delay"
//! in months caused by taking time off today.

use crate::experimental::runway_simulator::RunwaySimulator;

/// The result of a sabbatical analysis.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SabbaticalResult {
    /// Will you run out of money during the sabbatical?
    pub survives_sabbatical: bool,
    /// Months until you reach your goal if you DON'T take a sabbatical.
    pub months_to_goal_standard: u32,
    /// Months until you reach your goal if you DO take a sabbatical (includes the sabbatical time).
    pub months_to_goal_sabbatical: u32,
    /// The difference in months (how much the sabbatical delayed your retirement/goal).
    pub delay_months: u32,
}

/// A planner to evaluate the impact of taking a career break.
#[derive(Debug, Clone)]
pub struct SabbaticalPlanner {
    current_net_worth_cents: i64,
    target_net_worth_cents: i64,
    monthly_savings_cents: i64,
    annual_return_pct: f64,
    annual_inflation_pct: f64,
}

impl SabbaticalPlanner {
    /// Creates a new `SabbaticalPlanner`.
    #[must_use]
    pub const fn new(
        current_net_worth_cents: i64,
        target_net_worth_cents: i64,
        monthly_savings_cents: i64,
        annual_return_pct: f64,
        annual_inflation_pct: f64,
    ) -> Self {
        Self {
            current_net_worth_cents,
            target_net_worth_cents,
            monthly_savings_cents,
            annual_return_pct,
            annual_inflation_pct,
        }
    }

    /// Evaluates the impact of a sabbatical.
    ///
    /// # Arguments
    /// * `sabbatical_months` - How many months you want to take off.
    /// * `sabbatical_monthly_burn_cents` - How much you expect to spend per month during the break.
    #[must_use]
    #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
    pub fn evaluate(
        &self,
        sabbatical_months: u32,
        sabbatical_monthly_burn_cents: i64,
    ) -> SabbaticalResult {
        // Step 1: Check if the sabbatical itself is survivable.
        let runway_sim = RunwaySimulator::new(
            self.current_net_worth_cents,
            sabbatical_monthly_burn_cents,
            self.annual_inflation_pct,
        );
        let runway = runway_sim.calculate_runway();
        let survives_sabbatical = runway.months >= sabbatical_months;

        if !survives_sabbatical || self.current_net_worth_cents >= self.target_net_worth_cents {
            // Either they go broke, or they are already done.
            return SabbaticalResult {
                survives_sabbatical,
                months_to_goal_standard: 0,
                months_to_goal_sabbatical: 0,
                delay_months: 0,
            };
        }

        let monthly_return = self.annual_return_pct / 100.0 / 12.0;

        let standard_months = self.months_to_target(
            self.current_net_worth_cents,
            self.monthly_savings_cents,
            monthly_return,
        );

        // Sabbatical timeline:
        // First, apply drawdown and growth during the sabbatical.
        let mut post_sabbatical_nw = self.current_net_worth_cents;
        for _ in 0..sabbatical_months {
            post_sabbatical_nw -= sabbatical_monthly_burn_cents;
            // Compound whatever is left
            let gain = (post_sabbatical_nw as f64) * monthly_return;
            post_sabbatical_nw += gain.round() as i64;
        }

        let post_sabbatical_months = self.months_to_target(
            post_sabbatical_nw,
            self.monthly_savings_cents,
            monthly_return,
        );
        let total_sabbatical_timeline = sabbatical_months + post_sabbatical_months;

        SabbaticalResult {
            survives_sabbatical: true,
            months_to_goal_standard: standard_months,
            months_to_goal_sabbatical: total_sabbatical_timeline,
            delay_months: total_sabbatical_timeline.saturating_sub(standard_months),
        }
    }

    #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
    fn months_to_target(
        &self,
        initial_cents: i64,
        monthly_savings: i64,
        monthly_return: f64,
    ) -> u32 {
        let mut current = initial_cents;
        let mut months = 0;

        // Cap at 1200 months (100 years) to prevent infinite loops
        while current < self.target_net_worth_cents && months < 1200 {
            let gain = (current as f64) * monthly_return;
            current += gain.round() as i64;
            current += monthly_savings;
            months += 1;
        }
        months
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sabbatical_evaluation() {
        let planner = SabbaticalPlanner::new(
            10_000_000,  // $100k current
            100_000_000, // $1M target
            200_000,     // $2k/mo savings
            7.0,         // 7% return
            3.0,         // 3% inflation for runway
        );

        // Take 6 months off, spending $3k/mo
        let result = planner.evaluate(6, 300_000);

        assert!(result.survives_sabbatical);
        // It should take some number of months standard
        assert!(result.months_to_goal_standard > 0);
        assert!(result.months_to_goal_sabbatical > result.months_to_goal_standard);

        // The delay should be > 6 months because we missed out on 6 months of savings and growth
        assert!(result.delay_months > 6);
    }
}
