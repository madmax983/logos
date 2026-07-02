#![cfg(feature = "nova")]

//! Debt Life Energy Evaluator
//!
//! 🌟 Nova Mashup: Combining `DebtOptimizer` and `TrueWageCalculator` to answer the question:
//! "How many hours of your life are you trading away just to pay the interest on your debt?"

use crate::experimental::debt_optimizer::{DebtOptimizer, PayoffStrategy};
use crate::experimental::life_energy_calculator::TrueWageCalculator;

/// A report detailing the life energy cost of a specific debt payoff strategy.
#[derive(Debug, Clone, PartialEq)]
pub struct DebtLifeEnergyReport {
    /// The payoff strategy evaluated.
    pub strategy: PayoffStrategy,
    /// Total number of months to become debt free.
    pub total_months: u32,
    /// Total amount of interest paid in cents.
    pub total_interest_paid_cents: i64,
    /// The true hourly wage in cents.
    pub true_hourly_wage_cents: i64,
    /// The number of hours of your life spent working *just* to pay the interest.
    pub interest_life_energy_hours: f64,
}

/// Evaluates debt payoff strategies by calculating the life energy cost of interest.
#[derive(Debug, Clone)]
pub struct DebtLifeEnergyEvaluator {
    wage_calculator: TrueWageCalculator,
    debt_optimizer: DebtOptimizer,
}

impl DebtLifeEnergyEvaluator {
    /// Creates a new `DebtLifeEnergyEvaluator`.
    #[must_use]
    pub const fn new(wage_calculator: TrueWageCalculator, debt_optimizer: DebtOptimizer) -> Self {
        Self {
            wage_calculator,
            debt_optimizer,
        }
    }

    /// Evaluates a specific debt payoff strategy and returns a report including
    /// the life energy cost of the total interest paid.
    #[must_use]
    pub fn evaluate_strategy(&self, strategy: PayoffStrategy) -> DebtLifeEnergyReport {
        let payoff_result = self.debt_optimizer.simulate(strategy);

        let interest_hours = self
            .wage_calculator
            .evaluate_expense(payoff_result.total_interest_paid_cents);

        DebtLifeEnergyReport {
            strategy,
            total_months: payoff_result.total_months,
            total_interest_paid_cents: payoff_result.total_interest_paid_cents,
            true_hourly_wage_cents: self.wage_calculator.true_hourly_wage_cents(),
            interest_life_energy_hours: interest_hours,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::experimental::debt_optimizer::Debt;

    #[test]
    fn test_debt_life_energy_evaluation() {
        // $50/hr nominal, 40 hours a week, 5 hours commute, $100/week expenses = $42.22 true wage
        let wage_calc = TrueWageCalculator::new(50_00, 40.0, 5.0, 10_000);

        let mut debt_opt = DebtOptimizer::new(100_000); // $1,000/month budget
        debt_opt.add_debt(Debt {
            name: "Credit Card".to_string(),
            balance_cents: 500_000, // $5000
            interest_rate_pct: 20,
            min_payment_cents: 10_000, // $100
        });

        let evaluator = DebtLifeEnergyEvaluator::new(wage_calc, debt_opt);
        let report = evaluator.evaluate_strategy(PayoffStrategy::Avalanche);

        assert_eq!(report.strategy, PayoffStrategy::Avalanche);
        assert!(report.total_months > 0);
        assert!(report.total_interest_paid_cents > 0);
        assert_eq!(report.true_hourly_wage_cents, 42_22);

        // We are asserting that some non-zero amount of life energy was spent on interest
        assert!(report.interest_life_energy_hours > 0.0);
    }
}
