#![cfg(feature = "nova")]

//! Debt Runway Analyzer Module
//!
//! A mashup of `RunwaySimulator` and `Debt`. Analyzes if it is more beneficial for your
//! financial runway to hold onto your cash and pay the minimums, or wipe out the debt completely
//! to lower your monthly burn rate.

use crate::experimental::debt_optimizer::Debt;
use crate::experimental::runway_simulator::RunwaySimulator;

/// The result of comparing baseline runway vs. payoff runway.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebtRunwayResult {
    /// Number of months the runway lasts if you just pay the minimums.
    pub baseline_months: u32,
    /// Number of months the runway lasts if you pay off the debt upfront.
    pub payoff_months: u32,
    /// The absolute difference in months between the two scenarios.
    pub runway_difference_months: u32,
    /// True if paying off the debt yields a longer (or equal) runway.
    pub payoff_is_optimal: bool,
}

/// Analyzes the runway impact of carrying vs. paying off debt.
#[derive(Debug, Clone)]
pub struct DebtRunwayAnalyzer {
    liquid_assets_cents: i64,
    monthly_burn_without_debt_cents: i64,
    annual_inflation_pct: f64,
    debts: Vec<Debt>,
}

impl DebtRunwayAnalyzer {
    /// Creates a new `DebtRunwayAnalyzer`.
    ///
    /// # Arguments
    /// * `liquid_assets_cents` - Total available liquid assets in cents.
    /// * `monthly_burn_without_debt_cents` - Monthly expenses excluding minimum debt payments.
    /// * `annual_inflation_pct` - Expected annual inflation rate.
    #[must_use]
    pub const fn new(
        liquid_assets_cents: i64,
        monthly_burn_without_debt_cents: i64,
        annual_inflation_pct: f64,
    ) -> Self {
        Self {
            liquid_assets_cents,
            monthly_burn_without_debt_cents,
            annual_inflation_pct,
            debts: Vec::new(),
        }
    }

    /// Adds a debt to the analysis.
    pub fn add_debt(&mut self, debt: Debt) {
        self.debts.push(debt);
    }

    /// Evaluates whether it's better to hold cash or pay off debt.
    #[must_use]
    pub fn evaluate(&self) -> DebtRunwayResult {
        let mut total_debt_balance = 0;
        let mut total_min_payments = 0;

        for debt in &self.debts {
            total_debt_balance += debt.balance_cents;
            total_min_payments += debt.min_payment_cents;
        }

        // 1. Baseline Runway: Keep the cash, pay the minimums.
        let baseline_sim = RunwaySimulator::new(
            self.liquid_assets_cents,
            self.monthly_burn_without_debt_cents + total_min_payments,
            self.annual_inflation_pct,
        );
        let baseline_result = baseline_sim.calculate_runway();

        // 2. Payoff Runway: Spend cash to wipe debt, lower monthly burn.
        let remaining_assets = if total_debt_balance > self.liquid_assets_cents {
            0
        } else {
            self.liquid_assets_cents - total_debt_balance
        };

        let payoff_sim = RunwaySimulator::new(
            remaining_assets,
            self.monthly_burn_without_debt_cents,
            self.annual_inflation_pct,
        );
        let payoff_result = payoff_sim.calculate_runway();

        let payoff_is_optimal = payoff_result.months >= baseline_result.months;
        let runway_difference_months = if payoff_is_optimal {
            payoff_result.months - baseline_result.months
        } else {
            baseline_result.months - payoff_result.months
        };

        DebtRunwayResult {
            baseline_months: baseline_result.months,
            payoff_months: payoff_result.months,
            runway_difference_months,
            payoff_is_optimal,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::experimental::debt_optimizer::Debt;

    #[test]
    fn test_payoff_optimal() {
        let mut analyzer = DebtRunwayAnalyzer::new(10_000_000, 300_000, 0.0);

        analyzer.add_debt(Debt {
            name: "Car Loan".to_string(),
            balance_cents: 1_000_000,
            interest_rate_pct: 5,
            min_payment_cents: 100_000,
        });

        let result = analyzer.evaluate();

        // Baseline: 100k / (3k + 1k) = 25 months (25 * 4 = 100) -> 25
        // Payoff: (100k - 10k) / 3k = 90k / 3k = 30 months -> 30
        assert!(result.payoff_is_optimal);
        assert_eq!(result.baseline_months, 25);
        assert_eq!(result.payoff_months, 30);
        assert_eq!(result.runway_difference_months, 5);
    }

    #[test]
    fn test_keeping_cash_optimal() {
        let mut analyzer = DebtRunwayAnalyzer::new(2_000_000, 300_000, 0.0);

        analyzer.add_debt(Debt {
            name: "Student Loan".to_string(),
            balance_cents: 1_500_000,
            interest_rate_pct: 2,
            min_payment_cents: 10_000,
        });

        let result = analyzer.evaluate();

        // Baseline: 20k / 3.1k = 6.45 months, which means 7 partial/full months.
        // Payoff: (20k - 15k) / 3k = 5k / 3k = 1.66 months, which means 2 partial/full months.
        assert!(!result.payoff_is_optimal);
        assert_eq!(result.baseline_months, 7);
        assert_eq!(result.payoff_months, 2);
        assert_eq!(result.runway_difference_months, 5);
    }
}
