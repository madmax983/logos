#![cfg(feature = "nova")]

//! FIRE Delay Calculator
//!
//! Calculates how much a recurring expense delays your FIRE (Financial Independence, Retire Early) date.
//!
//! 🌟 Nova Mashup: We mash up the `NetWorthProjector` and `FireSimulator`
//! to show you exactly how many months of freedom you are trading for that monthly subscription.

use crate::planning::fire::{FireSimulator, UpcomingVest};
use crate::planning::net_worth_projector::NetWorthProjector;

/// Calculates the impact of an expense on your FIRE date.
#[derive(Debug, Clone)]
pub struct FireDelayCalculator {
    initial_net_worth_cents: i64,
    monthly_savings_cents: i64,
    monthly_expenses_cents: i64,
    upcoming_vests: Vec<UpcomingVest>,
}

impl FireDelayCalculator {
    /// Creates a new `FireDelayCalculator`.
    #[must_use]
    pub const fn new(
        initial_net_worth_cents: i64,
        monthly_savings_cents: i64,
        monthly_expenses_cents: i64,
    ) -> Self {
        Self {
            initial_net_worth_cents,
            monthly_savings_cents,
            monthly_expenses_cents,
            upcoming_vests: Vec::new(),
        }
    }

    /// Adds an upcoming RSU vest to the projection.
    pub fn add_upcoming_vest(&mut self, vest: UpcomingVest) {
        self.upcoming_vests.push(vest);
    }

    /// Calculates how many months a recurring expense delays your FIRE date.
    ///
    /// # Returns
    /// * `Some(delay_months)` if we could calculate the difference.
    /// * `None` if FIRE is never reached within the `max_months` projection.
    #[must_use]
    pub fn calculate_delay_months(&self, expense_cost_cents: i64, max_months: u16) -> Option<u16> {
        // Baseline (with the expense)
        // The expense is already assumed to be part of the `monthly_expenses_cents`
        // and its cost is absent from `monthly_savings_cents`.
        let baseline_fire_number = {
            let sim = FireSimulator::new(self.monthly_expenses_cents);
            sim.fire_number_cents()
        };

        let mut baseline_proj =
            NetWorthProjector::new(self.initial_net_worth_cents, self.monthly_savings_cents);
        baseline_proj.add_milestone_cents(baseline_fire_number);
        for vest in &self.upcoming_vests {
            baseline_proj.add_upcoming_vest(*vest);
        }

        let (_, baseline_crossed) = baseline_proj.project_timeline(max_months);
        let baseline_months_to_fire = baseline_crossed.first().map(|(_, m)| *m)?;

        // Optimized (without the expense)
        // If we cut the expense, our expenses go down, lowering our FIRE target.
        // Also, our savings go up by the expense amount.
        let optimized_expenses = self
            .monthly_expenses_cents
            .saturating_sub(expense_cost_cents);
        let optimized_savings = self
            .monthly_savings_cents
            .saturating_add(expense_cost_cents);

        let optimized_fire_number = {
            let sim = FireSimulator::new(optimized_expenses);
            sim.fire_number_cents()
        };

        let mut optimized_proj =
            NetWorthProjector::new(self.initial_net_worth_cents, optimized_savings);
        optimized_proj.add_milestone_cents(optimized_fire_number);
        for vest in &self.upcoming_vests {
            optimized_proj.add_upcoming_vest(*vest);
        }

        let (_, optimized_crossed) = optimized_proj.project_timeline(max_months);
        let optimized_months_to_fire = optimized_crossed.first().map(|(_, m)| *m)?;

        Some(baseline_months_to_fire.saturating_sub(optimized_months_to_fire))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fire_delay_calculator() {
        // Initial NW: $100k, Savings: $2k/mo, Expenses: $4k/mo
        let calc = FireDelayCalculator::new(10_000_000, 200_000, 400_000);

        // Let's see the delay of a $500/mo car payment
        // Max project: 1200 months (100 years)
        let delay = calc.calculate_delay_months(50_000, 1200);

        assert!(delay.is_some());
        let delay_months = delay.unwrap();

        // $500/mo reduction in expenses means FIRE number drops by $150k (500 * 12 * 25)
        // And savings increase by $500/mo. This should save a lot of months!
        assert!(delay_months > 0);
    }

    #[test]
    fn test_fire_delay_with_vests() {
        let mut calc = FireDelayCalculator::new(10_000_000, 200_000, 400_000);
        calc.add_upcoming_vest(UpcomingVest {
            avg_close_price_cents: 10_000,
            units: 1000,
            days_to_vest: 30,
        });

        let delay = calc.calculate_delay_months(50_000, 1200);
        assert!(delay.is_some());
        assert!(delay.unwrap() > 0);
    }

    #[test]
    fn test_no_fire_reached() {
        // Savings are negative, will never reach FIRE
        let calc = FireDelayCalculator::new(10_000, 0, 400_000);
        let delay = calc.calculate_delay_months(500, 120); // only 10 years
        assert_eq!(delay, None);
    }
}
