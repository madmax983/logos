#![cfg(feature = "nova")]

//! Debt Life Energy Evaluator
//!
//! 🌟 Nova Mashup: Combines `DebtOptimizer` and `TrueWageCalculator` to reveal
//! exactly how many hours of your life (Life Energy) you will spend working just
//! to pay off the *interest* on your debts, based on your chosen payoff strategy.

use crate::experimental::debt_optimizer::{DebtOptimizer, PayoffStrategy};
use crate::experimental::life_energy_calculator::TrueWageCalculator;

/// A report detailing the life energy cost of debt interest.
#[derive(Debug, Clone, PartialEq)]
pub struct DebtLifeEnergyReport {
    /// Total months to pay off the debt.
    pub total_months: u32,
    /// Total interest paid in cents.
    pub total_interest_paid_cents: i64,
    /// The true hourly wage in cents.
    pub true_hourly_wage_cents: i64,
    /// Hours of life energy spent purely on paying interest.
    pub interest_life_energy_hours: f64,
}

/// Evaluates the life energy cost of debt payoff strategies.
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

    /// Evaluates the life energy cost of the given payoff strategy.
    #[must_use]
    pub fn evaluate_strategy(&self, strategy: PayoffStrategy) -> DebtLifeEnergyReport {
        let payoff_result = self.debt_optimizer.simulate(strategy);
        let interest_hours = self
            .wage_calculator
            .evaluate_expense(payoff_result.total_interest_paid_cents);

        DebtLifeEnergyReport {
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
        let calc = TrueWageCalculator::new(50_00, 40.0, 5.0, 10_000); // true wage $42.22

        let mut optimizer = DebtOptimizer::new(100_000); // $1,000/month budget
        optimizer.add_debt(Debt {
            name: "Student Loan".to_string(),
            balance_cents: 500_000, // $5,000
            interest_rate_pct: 25,
            min_payment_cents: 10_000, // $100
        });

        let evaluator = DebtLifeEnergyEvaluator::new(calc, optimizer);

        let report = evaluator.evaluate_strategy(PayoffStrategy::Avalanche);

        assert!(report.total_months > 0);
        assert!(report.total_interest_paid_cents > 0);
        assert_eq!(report.true_hourly_wage_cents, 42_22);

        #[allow(clippy::cast_precision_loss)]
        let expected_hours = report.total_interest_paid_cents as f64 / 42_22.0;
        assert!((report.interest_life_energy_hours - expected_hours).abs() < 0.01);
        assert!((report.interest_life_energy_hours - expected_hours).abs() < 0.01);
    }
}
