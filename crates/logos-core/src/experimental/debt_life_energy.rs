#![cfg(feature = "nova")]

//! Debt Life Energy Evaluator
//!
//! A mashup of the Debt Optimizer and True Wage Calculator.
//! It translates the raw financial cost of debt (the interest paid over time)
//! into the ultimate metric: how many hours of your life you are spending
//! just to service that debt.

use crate::experimental::debt_optimizer::{DebtOptimizer, PayoffStrategy};
use crate::experimental::life_energy_calculator::TrueWageCalculator;

/// The life energy cost of a specific debt payoff strategy.
#[derive(Debug, Clone, PartialEq)]
pub struct StrategyLifeEnergyCost {
    /// The payoff strategy evaluated.
    pub strategy: PayoffStrategy,
    /// Total months to become debt-free.
    pub total_months: u32,
    /// Total interest paid in cents.
    pub total_interest_paid_cents: i64,
    /// Hours of life energy spent working purely to pay off the interest.
    pub life_energy_hours_spent_on_interest: f64,
}

/// A comparison between two payoff strategies measured in life energy.
#[derive(Debug, Clone, PartialEq)]
pub struct DebtLifeEnergyComparison {
    pub snowball: StrategyLifeEnergyCost,
    pub avalanche: StrategyLifeEnergyCost,
    /// How many hours of your life you save by choosing the faster strategy.
    pub hours_of_life_saved: f64,
}

/// Evaluates debt payoff strategies in terms of Life Energy (hours worked).
#[derive(Debug, Clone)]
pub struct DebtLifeEnergyEvaluator {
    optimizer: DebtOptimizer,
    wage_calculator: TrueWageCalculator,
}

impl DebtLifeEnergyEvaluator {
    #[must_use]
    pub const fn new(optimizer: DebtOptimizer, wage_calculator: TrueWageCalculator) -> Self {
        Self {
            optimizer,
            wage_calculator,
        }
    }

    /// Evaluates a single strategy and calculates its life energy cost.
    #[must_use]
    pub fn evaluate_strategy(&self, strategy: PayoffStrategy) -> StrategyLifeEnergyCost {
        let payoff_result = self.optimizer.simulate(strategy);
        let hours = self
            .wage_calculator
            .evaluate_expense(payoff_result.total_interest_paid_cents);

        StrategyLifeEnergyCost {
            strategy,
            total_months: payoff_result.total_months,
            total_interest_paid_cents: payoff_result.total_interest_paid_cents,
            life_energy_hours_spent_on_interest: hours,
        }
    }

    /// Compares Snowball and Avalanche strategies, showing life energy saved.
    #[must_use]
    pub fn compare_strategies(&self) -> DebtLifeEnergyComparison {
        let snowball = self.evaluate_strategy(PayoffStrategy::Snowball);
        let avalanche = self.evaluate_strategy(PayoffStrategy::Avalanche);

        // Assume avalanche is always mathematically faster/cheaper or equal
        let hours_saved = snowball.life_energy_hours_spent_on_interest
            - avalanche.life_energy_hours_spent_on_interest;

        DebtLifeEnergyComparison {
            snowball,
            avalanche,
            hours_of_life_saved: f64::max(0.0, hours_saved),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::experimental::debt_optimizer::Debt;

    #[test]
    fn test_debt_life_energy_comparison() {
        // True wage: $50/hr, 40 hours, 0 commute, 0 expenses = exactly $50.00/hr true wage (5000 cents)
        let wage_calc = TrueWageCalculator::new(50_00, 40.0, 0.0, 0);

        let mut optimizer = DebtOptimizer::new(100_000); // $1000/mo payment

        // Debt 1: Small balance, 10% interest
        optimizer.add_debt(Debt {
            name: "Credit Card".to_string(),
            balance_cents: 200_000,
            interest_rate_pct: 10,
            min_payment_cents: 5_000,
        });

        // Debt 2: Large balance, 25% interest
        optimizer.add_debt(Debt {
            name: "Student Loan".to_string(),
            balance_cents: 500_000,
            interest_rate_pct: 25,
            min_payment_cents: 10_000,
        });

        let evaluator = DebtLifeEnergyEvaluator::new(optimizer, wage_calc);
        let comparison = evaluator.compare_strategies();

        assert_eq!(comparison.snowball.strategy, PayoffStrategy::Snowball);
        assert_eq!(comparison.avalanche.strategy, PayoffStrategy::Avalanche);

        // Avalanche should save life energy compared to snowball
        assert!(comparison.hours_of_life_saved > 0.0);
        assert!(
            comparison.snowball.life_energy_hours_spent_on_interest
                > comparison.avalanche.life_energy_hours_spent_on_interest
        );
    }
}
