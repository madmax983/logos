//! Debt Life Energy Evaluator
//!
//! 🌟 Nova Mashup: Combines `DebtOptimizer` and `TrueWageCalculator` to evaluate
//! how many hours of your life are consumed by debt interest under different payoff strategies.

use crate::experimental::debt_optimizer::{DebtOptimizer, PayoffStrategy};
use crate::experimental::life_energy_calculator::TrueWageCalculator;

/// A report detailing the life energy cost of debt interest under different strategies.
#[derive(Debug, Clone, PartialEq)]
pub struct DebtLifeEnergyReport {
    pub snowball_interest_cents: i64,
    pub snowball_life_energy_hours: f64,
    pub avalanche_interest_cents: i64,
    pub avalanche_life_energy_hours: f64,
    pub life_energy_saved_hours: f64,
}

/// Evaluates the life energy consumed by debt interest under different payoff strategies.
#[derive(Debug, Clone)]
pub struct DebtLifeEnergyEvaluator {
    wage_calculator: TrueWageCalculator,
    debt_optimizer: DebtOptimizer,
}

impl DebtLifeEnergyEvaluator {
    #[must_use]
    pub const fn new(wage_calculator: TrueWageCalculator, debt_optimizer: DebtOptimizer) -> Self {
        Self {
            wage_calculator,
            debt_optimizer,
        }
    }

    #[must_use]
    pub fn evaluate(&self) -> DebtLifeEnergyReport {
        let snowball = self.debt_optimizer.simulate(PayoffStrategy::Snowball);
        let avalanche = self.debt_optimizer.simulate(PayoffStrategy::Avalanche);

        let snowball_hours = self
            .wage_calculator
            .evaluate_expense(snowball.total_interest_paid_cents);
        let avalanche_hours = self
            .wage_calculator
            .evaluate_expense(avalanche.total_interest_paid_cents);

        DebtLifeEnergyReport {
            snowball_interest_cents: snowball.total_interest_paid_cents,
            snowball_life_energy_hours: snowball_hours,
            avalanche_interest_cents: avalanche.total_interest_paid_cents,
            avalanche_life_energy_hours: avalanche_hours,
            life_energy_saved_hours: snowball_hours - avalanche_hours,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::experimental::debt_optimizer::Debt;

    #[test]
    fn test_debt_life_energy_evaluator() {
        let wage_calc = TrueWageCalculator::new(50_00, 40.0, 5.0, 10_000);
        let mut debt_opt = DebtOptimizer::new(100_000);

        debt_opt.add_debt(Debt {
            name: "Credit Card".to_string(),
            balance_cents: 200_000,
            interest_rate_pct: 10,
            min_payment_cents: 5_000,
        });

        debt_opt.add_debt(Debt {
            name: "Student Loan".to_string(),
            balance_cents: 500_000,
            interest_rate_pct: 25,
            min_payment_cents: 10_000,
        });

        let evaluator = DebtLifeEnergyEvaluator::new(wage_calc, debt_opt);
        let report = evaluator.evaluate();

        assert!(report.snowball_interest_cents > report.avalanche_interest_cents);
        assert!(report.snowball_life_energy_hours > report.avalanche_life_energy_hours);
        assert!(report.life_energy_saved_hours > 0.0);
    }
}
