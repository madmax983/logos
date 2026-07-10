#![cfg(feature = "nova")]

//! Debt vs Invest Analyzer
//!
//! A simulator that compares two strategies for dealing with debt:
//! 1. **Aggressive Payoff:** Put all extra cash towards the debt.
//! 2. **Invest the Difference:** Pay only the minimum on the debt and invest the rest.
//!
//! 🌟 Nova Mashup: We mash up `DebtOptimizer` concepts with `OpportunityCostAnalyzer` concepts!

use crate::experimental::debt_optimizer::Debt;

/// The outcome of the Debt vs Invest simulation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebtVsInvestResult {
    /// Total net worth at the end of the simulation if aggressively paying off the debt.
    pub aggressive_payoff_net_worth_cents: i64,
    /// Total net worth at the end of the simulation if investing the difference.
    pub invest_difference_net_worth_cents: i64,
    /// The optimal strategy string.
    pub optimal_strategy: String,
}

/// A simulator to compare debt payoff vs investing.
#[derive(Debug, Clone)]
pub struct DebtVsInvestAnalyzer {
    debt: Debt,
    monthly_budget_cents: i64,
    annual_investment_return_pct: f64,
    years: u8,
}

impl DebtVsInvestAnalyzer {
    /// Creates a new `DebtVsInvestAnalyzer`.
    ///
    /// # Arguments
    /// * `debt` - The debt obligation.
    /// * `monthly_budget_cents` - The total amount available each month for either debt payoff or investing.
    /// * `annual_investment_return_pct` - Expected annual return on investments (e.g., 7.0 for 7%).
    /// * `years` - How many years to project.
    #[must_use]
    pub const fn new(
        debt: Debt,
        monthly_budget_cents: i64,
        annual_investment_return_pct: f64,
        years: u8,
    ) -> Self {
        Self {
            debt,
            monthly_budget_cents,
            annual_investment_return_pct,
            years,
        }
    }

    /// Simulates the two strategies over the specified time horizon.
    #[must_use]
    #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
    pub fn simulate(&self) -> DebtVsInvestResult {
        let months = u32::from(self.years) * 12;
        let monthly_investment_rate = self.annual_investment_return_pct / 100.0 / 12.0;
        let monthly_debt_rate = f64::from(self.debt.interest_rate_pct) / 100.0 / 12.0;

        // Scenario 1: Aggressive Payoff
        let mut agg_debt_balance = self.debt.balance_cents as f64;
        let mut agg_investments = 0.0;

        for _ in 0..months {
            if agg_debt_balance > 0.0 {
                agg_debt_balance *= 1.0 + monthly_debt_rate;
                let payment = agg_debt_balance.min(self.monthly_budget_cents as f64);
                agg_debt_balance -= payment;
                let leftover = (self.monthly_budget_cents as f64) - payment;

                agg_investments *= 1.0 + monthly_investment_rate;
                agg_investments += leftover;
            } else {
                agg_investments *= 1.0 + monthly_investment_rate;
                agg_investments += self.monthly_budget_cents as f64;
            }
        }

        // Scenario 2: Invest the Difference
        let mut inv_debt_balance = self.debt.balance_cents as f64;
        let mut inv_investments = 0.0;

        for _ in 0..months {
            if inv_debt_balance > 0.0 {
                inv_debt_balance *= 1.0 + monthly_debt_rate;
                let payment = inv_debt_balance.min(self.debt.min_payment_cents as f64);
                inv_debt_balance -= payment;
                let leftover = (self.monthly_budget_cents as f64) - payment;

                inv_investments *= 1.0 + monthly_investment_rate;
                inv_investments += leftover.max(0.0);
            } else {
                inv_investments *= 1.0 + monthly_investment_rate;
                inv_investments += self.monthly_budget_cents as f64;
            }
        }

        let agg_net_worth = (agg_investments - agg_debt_balance).round() as i64;
        let inv_net_worth = (inv_investments - inv_debt_balance).round() as i64;

        let optimal_strategy = if agg_net_worth > inv_net_worth {
            "Aggressive Payoff".to_string()
        } else {
            "Invest the Difference".to_string()
        };

        DebtVsInvestResult {
            aggressive_payoff_net_worth_cents: agg_net_worth,
            invest_difference_net_worth_cents: inv_net_worth,
            optimal_strategy,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_high_interest_debt_favors_payoff() {
        let debt = Debt {
            name: "Credit Card".to_string(),
            balance_cents: 1_000_000,  // $10,000
            interest_rate_pct: 20,     // 20% interest
            min_payment_cents: 20_000, // $200 min payment
        };

        let analyzer = DebtVsInvestAnalyzer::new(debt, 100_000, 7.0, 10);
        let result = analyzer.simulate();

        assert_eq!(result.optimal_strategy, "Aggressive Payoff");
        assert!(
            result.aggressive_payoff_net_worth_cents > result.invest_difference_net_worth_cents
        );
    }

    #[test]
    fn test_low_interest_debt_favors_investing() {
        let debt = Debt {
            name: "Mortgage".to_string(),
            balance_cents: 1_000_000,  // $10,000
            interest_rate_pct: 3,      // 3% interest
            min_payment_cents: 20_000, // $200 min payment
        };

        let analyzer = DebtVsInvestAnalyzer::new(debt, 100_000, 7.0, 10);
        let result = analyzer.simulate();

        assert_eq!(result.optimal_strategy, "Invest the Difference");
        assert!(
            result.invest_difference_net_worth_cents > result.aggressive_payoff_net_worth_cents
        );
    }
}
