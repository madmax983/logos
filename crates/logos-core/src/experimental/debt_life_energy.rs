#![cfg(feature = "nova")]

//! Debt Life Energy Calculator
//!
//! Combines the `DebtOptimizer` and `TrueWageCalculator` to determine the visceral
//! "life energy" cost of debt. Instead of just seeing the monetary cost of interest,
//! you see how many hours of your actual life you are trading to service that debt.

use crate::experimental::debt_optimizer::{DebtOptimizer, PayoffStrategy};
use crate::experimental::life_energy_calculator::TrueWageCalculator;

/// The result of evaluating a debt payoff strategy through the lens of life energy.
#[derive(Debug, Clone, PartialEq)]
pub struct DebtLifeEnergyResult {
    /// The total number of months to pay off the debt.
    pub total_months: u32,
    /// The total amount of interest paid in cents.
    pub total_interest_paid_cents: i64,
    /// The number of life energy hours spent just to pay the interest.
    pub interest_life_energy_hours: f64,
}

/// Calculates the life energy cost of different debt payoff strategies.
#[derive(Debug, Clone)]
pub struct DebtLifeEnergyCalculator {
    debt_optimizer: DebtOptimizer,
    wage_calculator: TrueWageCalculator,
}

impl DebtLifeEnergyCalculator {
    /// Creates a new `DebtLifeEnergyCalculator`.
    #[must_use]
    pub const fn new(debt_optimizer: DebtOptimizer, wage_calculator: TrueWageCalculator) -> Self {
        Self {
            debt_optimizer,
            wage_calculator,
        }
    }

    /// Evaluates a specific debt payoff strategy and returns the life energy cost.
    #[must_use]
    pub fn evaluate_debt_strategy(&self, strategy: PayoffStrategy) -> DebtLifeEnergyResult {
        let payoff_result = self.debt_optimizer.simulate(strategy);
        let interest_hours = self
            .wage_calculator
            .evaluate_expense(payoff_result.total_interest_paid_cents);

        DebtLifeEnergyResult {
            total_months: payoff_result.total_months,
            total_interest_paid_cents: payoff_result.total_interest_paid_cents,
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
        let mut optimizer = DebtOptimizer::new(100_000); // $1000/mo

        optimizer.add_debt(Debt {
            name: "Credit Card".to_string(),
            balance_cents: 500_000, // $5000
            interest_rate_pct: 20,
            min_payment_cents: 10_000, // $100
        });

        // True wage: $50/hr nominal, 40 hrs/wk, 5 hrs commute, $100/wk expenses
        // Nominal weekly: 2000
        // True weekly: 1900
        // Total hours: 45
        // True hourly wage: 1900 / 45 = $42.22 / hr (4222 cents)
        let wage_calc = TrueWageCalculator::new(50_00, 40.0, 5.0, 10_000);

        let calculator = DebtLifeEnergyCalculator::new(optimizer, wage_calc);

        let result = calculator.evaluate_debt_strategy(PayoffStrategy::Avalanche);

        assert!(result.total_months > 0);
        assert!(result.total_interest_paid_cents > 0);

        // Check that the hours match the interest / true wage
        #[allow(clippy::cast_precision_loss)]
        let expected_hours = result.total_interest_paid_cents as f64 / 4222.0;
        assert!((result.interest_life_energy_hours - expected_hours).abs() < 0.01);
    }
}
