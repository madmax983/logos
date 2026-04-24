//! Life Energy Calculator
//!
//! Inspired by "Your Money or Your Life", this module calculates the "True Hourly Wage"
//! by factoring in job-related expenses and commute time, then converts financial
//! expenses into "Life Energy" (hours of your life spent to buy the item).
//!
//! 🌟 Nova Mashup: We mash this up with the `OpportunityCostAnalyzer` so you can see
//! not just the direct life energy cost of a subscription, but the long-term
//! life energy opportunity cost of holding it.

use crate::experimental::opportunity_cost::OpportunityCostAnalyzer;
use crate::experimental::cashflow_projector::RecurringTemplate;

/// Calculates the true hourly wage, accounting for hidden job costs.
#[derive(Debug, Clone)]
pub struct TrueWageCalculator {
    nominal_hourly_wage_cents: i64,
    weekly_hours_worked: f64,
    weekly_commute_hours: f64,
    weekly_job_expenses_cents: i64,
}

impl TrueWageCalculator {
    #[must_use]
    pub const fn new(
        nominal_hourly_wage_cents: i64,
        weekly_hours_worked: f64,
        weekly_commute_hours: f64,
        weekly_job_expenses_cents: i64,
    ) -> Self {
        Self {
            nominal_hourly_wage_cents,
            weekly_hours_worked,
            weekly_commute_hours,
            weekly_job_expenses_cents,
        }
    }

    /// Calculates the effective hourly wage in cents after accounting for commute time
    /// and job-related expenses.
    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    pub fn true_hourly_wage_cents(&self) -> i64 {
        let nominal_weekly_pay = (self.nominal_hourly_wage_cents as f64 * self.weekly_hours_worked) as i64;
        let true_weekly_pay = nominal_weekly_pay - self.weekly_job_expenses_cents;
        let total_hours_committed = self.weekly_hours_worked + self.weekly_commute_hours;

        if total_hours_committed <= 0.0 {
            return 0;
        }

        (true_weekly_pay as f64 / total_hours_committed).round() as i64
    }

    /// Evaluates how many hours of life energy a single expense costs.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn evaluate_expense(&self, expense_cents: i64) -> f64 {
        let true_wage = self.true_hourly_wage_cents();
        if true_wage <= 0 {
            return f64::INFINITY; // Infinite life energy cost if you don't make money
        }

        expense_cents as f64 / true_wage as f64
    }
}

/// A report detailing the life energy cost of a recurring subscription.
#[derive(Debug, Clone, PartialEq)]
pub struct LifeEnergySubscriptionReport {
    pub description: String,
    pub true_hourly_wage_cents: i64,
    pub direct_monthly_hours: f64,
    pub opportunity_cost_future_value_cents: i64,
    pub opportunity_cost_future_hours: f64,
}

/// Evaluates recurring subscriptions by mashing up `TrueWageCalculator` and `OpportunityCostAnalyzer`.
#[derive(Debug, Clone)]
pub struct LifeEnergySubscriptionEvaluator {
    wage_calculator: TrueWageCalculator,
    opportunity_analyzer: OpportunityCostAnalyzer,
}

impl LifeEnergySubscriptionEvaluator {
    #[must_use]
    pub const fn new(wage_calculator: TrueWageCalculator, opportunity_analyzer: OpportunityCostAnalyzer) -> Self {
        Self {
            wage_calculator,
            opportunity_analyzer,
        }
    }

    /// Analyzes a recurring subscription to find its long-term life energy cost.
    #[must_use]
    pub fn evaluate_subscription(&self, template: &RecurringTemplate) -> LifeEnergySubscriptionReport {
        let opp_cost = self.opportunity_analyzer.analyze(template);
        let direct_hours = self.wage_calculator.evaluate_expense(template.amount_cents);
        let future_hours = self.wage_calculator.evaluate_expense(opp_cost.future_value_cents);

        LifeEnergySubscriptionReport {
            description: template.description.clone(),
            true_hourly_wage_cents: self.wage_calculator.true_hourly_wage_cents(),
            direct_monthly_hours: direct_hours,
            opportunity_cost_future_value_cents: opp_cost.future_value_cents,
            opportunity_cost_future_hours: future_hours,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_true_wage_calculation() {
        // $50/hr nominal, 40 hours a week
        // 5 hours commute/week
        // $100/week on gas/job expenses
        let calc = TrueWageCalculator::new(50_00, 40.0, 5.0, 100_00);

        // Nominal weekly: $2000
        // True weekly: $1900
        // Total hours: 45
        // True wage: 1900 / 45 = $42.22 / hr
        assert_eq!(calc.true_hourly_wage_cents(), 42_22);
    }

    #[test]
    fn test_evaluate_expense() {
        let calc = TrueWageCalculator::new(50_00, 40.0, 5.0, 100_00); // true wage $42.22

        // $42.22 expense should cost ~1 hour
        let hours = calc.evaluate_expense(42_22);
        assert!((hours - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_subscription_opportunity_mashup() {
        let calc = TrueWageCalculator::new(50_00, 40.0, 5.0, 100_00); // true wage $42.22
        let opp_analyzer = OpportunityCostAnalyzer::new(7.0, 10);

        let evaluator = LifeEnergySubscriptionEvaluator::new(calc, opp_analyzer);

        let template = RecurringTemplate {
            description: "Streaming".to_string(),
            amount_cents: 15_99, // $15.99/mo
            credit_account: "assets:checking".to_string(),
            debit_account: "expenses:entertainment".to_string(),
        };

        let report = evaluator.evaluate_subscription(&template);

        assert_eq!(report.description, "Streaming");
        assert_eq!(report.true_hourly_wage_cents, 42_22);

        // direct cost is 15.99 / 42.22 = 0.378 hours per month
        assert!((report.direct_monthly_hours - 0.378).abs() < 0.01);

        // from opp cost tests we know FV is ~276_880 ($2768.80)
        // future hours = 2768.80 / 42.22 = ~65.5 hours
        assert!((report.opportunity_cost_future_hours - 65.5).abs() < 1.0);
    }
}
