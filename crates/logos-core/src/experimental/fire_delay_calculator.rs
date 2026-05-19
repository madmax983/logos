//! FIRE Delay Calculator Module.
//!
//! Answers the question: "How many months will this expense push back my FIRE date?"
//!
//! This module mashes up the `OpportunityCostAnalyzer` and `FireAscentSimulator`
//! to calculate exactly how much extra time you'll need to work because of a
//! recurring subscription or a one-time large purchase.

use crate::experimental::cashflow_projector::RecurringTemplate;
use crate::experimental::fire_ascent::FireAscentSimulator;
use crate::experimental::opportunity_cost::OpportunityCostAnalyzer;
use crate::planning::fire::FireSimulator;
use crate::planning::net_worth_projector::NetWorthProjector;

/// The result of calculating the delay to FIRE caused by an expense.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FireDelayResult {
    /// The number of months it originally took to reach FIRE.
    pub original_months_to_fire: Option<u16>,
    /// The number of months it now takes to reach FIRE with the expense.
    pub new_months_to_fire: Option<u16>,
    /// The delay in months caused by the expense.
    pub delay_months: Option<u16>,
    /// The future value opportunity cost of the expense in cents.
    pub opportunity_cost_cents: i64,
}

/// A calculator to determine the time cost of expenses.
#[derive(Debug, Clone)]
pub struct FireDelayCalculator {
    fire_sim: FireSimulator,
    initial_net_worth_cents: i64,
    monthly_savings_cents: i64,
    max_months: u16,
    opportunity_analyzer: OpportunityCostAnalyzer,
}

impl FireDelayCalculator {
    /// Creates a new `FireDelayCalculator`.
    #[must_use]
    pub const fn new(
        fire_sim: FireSimulator,
        initial_net_worth_cents: i64,
        monthly_savings_cents: i64,
        max_months: u16,
        opportunity_analyzer: OpportunityCostAnalyzer,
    ) -> Self {
        Self {
            fire_sim,
            initial_net_worth_cents,
            monthly_savings_cents,
            max_months,
            opportunity_analyzer,
        }
    }

    fn get_months_to_fire(
        fire_sim: &FireSimulator,
        projector: &NetWorthProjector,
        max_months: u16,
    ) -> Option<u16> {
        let ascent_sim = FireAscentSimulator::new(fire_sim.clone(), projector.clone(), max_months);
        let result = ascent_sim.ascend();
        if result.success {
            result.milestones.last().and_then(|m| m.month_reached)
        } else {
            None
        }
    }

    /// Calculates the delay caused by a recurring expense.
    #[must_use]
    pub fn calculate_recurring_delay(&self, template: &RecurringTemplate) -> FireDelayResult {
        let projector =
            NetWorthProjector::new(self.initial_net_worth_cents, self.monthly_savings_cents);
        let original_months = Self::get_months_to_fire(&self.fire_sim, &projector, self.max_months);

        let opp_result = self.opportunity_analyzer.analyze(template);

        let new_savings = self
            .monthly_savings_cents
            .saturating_sub(template.amount_cents);
        let new_projector = NetWorthProjector::new(self.initial_net_worth_cents, new_savings);

        let new_months = Self::get_months_to_fire(&self.fire_sim, &new_projector, self.max_months);

        let delay = match (original_months, new_months) {
            (Some(orig), Some(new)) => Some(new.saturating_sub(orig)),
            _ => None,
        };

        FireDelayResult {
            original_months_to_fire: original_months,
            new_months_to_fire: new_months,
            delay_months: delay,
            opportunity_cost_cents: opp_result.future_value_cents,
        }
    }

    /// Calculates the delay caused by a one-time expense.
    #[must_use]
    pub fn calculate_one_time_delay(&self, amount_cents: i64) -> FireDelayResult {
        let projector =
            NetWorthProjector::new(self.initial_net_worth_cents, self.monthly_savings_cents);
        let original_months = Self::get_months_to_fire(&self.fire_sim, &projector, self.max_months);

        let new_initial_nw = self.initial_net_worth_cents.saturating_sub(amount_cents);
        let new_projector = NetWorthProjector::new(new_initial_nw, self.monthly_savings_cents);

        let new_months = Self::get_months_to_fire(&self.fire_sim, &new_projector, self.max_months);

        let delay = match (original_months, new_months) {
            (Some(orig), Some(new)) => Some(new.saturating_sub(orig)),
            _ => None,
        };

        FireDelayResult {
            original_months_to_fire: original_months,
            new_months_to_fire: new_months,
            delay_months: delay,
            opportunity_cost_cents: amount_cents, // For one-time, opportunity cost is at least the amount itself, though realistically it's more if invested. We return the base amount here.
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::planning::fire::FireConfig;

    #[test]
    fn test_calculate_recurring_delay() {
        let mut fire_sim = FireSimulator::new(400_000); // 4k/mo expenses = 48k/yr. Fire number = 1.2M
        fire_sim.set_config(FireConfig {
            safe_withdrawal_rate_pct: 4,
        });

        let initial_nw = 80_000_000; // 800k. Needs 400k.
        let savings = 1_000_000; // 10k/mo. 400k / 10k = 40 months.

        let opp_analyzer = OpportunityCostAnalyzer::new(7.0, 10);
        let calc = FireDelayCalculator::new(fire_sim, initial_nw, savings, 120, opp_analyzer);

        // A $2,000/mo expense!
        let template = RecurringTemplate {
            description: "Luxury Apartment".to_string(),
            amount_cents: 200_000,
            credit_account: "assets:checking".to_string(),
            debit_account: "expenses:rent".to_string(),
        };

        let result = calc.calculate_recurring_delay(&template);

        assert_eq!(result.original_months_to_fire, Some(40));

        // New savings: 8k/mo. 400k / 8k = 50 months.
        assert_eq!(result.new_months_to_fire, Some(50));
        assert_eq!(result.delay_months, Some(10));
    }

    #[test]
    fn test_calculate_one_time_delay() {
        let mut fire_sim = FireSimulator::new(400_000); // 4k/mo expenses = 48k/yr. Fire number = 1.2M
        fire_sim.set_config(FireConfig {
            safe_withdrawal_rate_pct: 4,
        });

        let initial_nw = 80_000_000; // 800k. Needs 400k.
        let savings = 1_000_000; // 10k/mo. 400k / 10k = 40 months.

        let opp_analyzer = OpportunityCostAnalyzer::new(7.0, 10);
        let calc = FireDelayCalculator::new(fire_sim, initial_nw, savings, 120, opp_analyzer);

        // A $50,000 car!
        let result = calc.calculate_one_time_delay(5_000_000);

        assert_eq!(result.original_months_to_fire, Some(40));

        // New initial NW: 750k. Needs 450k. 450k / 10k = 45 months.
        assert_eq!(result.new_months_to_fire, Some(45));
        assert_eq!(result.delay_months, Some(5));
    }
}
