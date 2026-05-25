#![cfg(feature = "nova")]

//! Interest Rate Arbitrage Simulator
//!
//! A tool to determine whether it is mathematically optimal to aggressively pay off
//! low-interest debt or to pay the minimum and invest the difference.

use crate::experimental::debt_optimizer::Debt;

/// The strategy that yields the higher net worth.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OptimalStrategy {
    /// Paying off the debt as fast as possible yields a higher net worth.
    PayDebtAggressively,
    /// Paying the minimum and investing the rest yields a higher net worth.
    InvestDifference,
    /// Both strategies yield the same net worth.
    Neutral,
}

/// The result of an arbitrage simulation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArbitrageResult {
    /// Net worth after the simulation period if paying debt aggressively.
    pub aggressive_debt_net_worth_cents: i64,
    /// Net worth if paying the minimum and investing the difference.
    pub invest_difference_net_worth_cents: i64,
    /// The optimal strategy recommendation.
    pub optimal_strategy: OptimalStrategy,
}

/// Simulates the net worth impact of different debt payoff strategies.
#[derive(Debug, Clone)]
pub struct ArbitrageSimulator {
    debt: Debt,
    monthly_budget_cents: i64,
    annual_investment_return_pct: f64,
    simulation_months: u32,
}

impl ArbitrageSimulator {
    /// Creates a new `ArbitrageSimulator`.
    ///
    /// # Arguments
    /// * `debt` - The debt obligation to analyze.
    /// * `monthly_budget_cents` - The total monthly cash flow available for debt + investing. Must be >= minimum payment.
    /// * `annual_investment_return_pct` - Expected annual return on investments (e.g., 7.0 for 7%).
    /// * `simulation_months` - The time horizon to simulate.
    #[must_use]
    pub const fn new(
        debt: Debt,
        monthly_budget_cents: i64,
        annual_investment_return_pct: f64,
        simulation_months: u32,
    ) -> Self {
        Self {
            debt,
            monthly_budget_cents,
            annual_investment_return_pct,
            simulation_months,
        }
    }

    /// Simulates the two strategies and returns the comparison.
    #[must_use]
    pub fn simulate(&self) -> ArbitrageResult {
        let mut aggressive_investments = 0_f64;
        let mut aggressive_debt_balance = self.debt.balance_cents;

        let mut invest_investments = 0_f64;
        let mut invest_debt_balance = self.debt.balance_cents;

        let monthly_invest_rate = self.annual_investment_return_pct / 100.0 / 12.0;

        for _ in 0..self.simulation_months {
            // Path 1: Aggressive Debt Payoff
            let aggressive_interest =
                (aggressive_debt_balance * i64::from(self.debt.interest_rate_pct)) / 100 / 12;
            aggressive_debt_balance += aggressive_interest;

            let aggressive_payment =
                std::cmp::min(aggressive_debt_balance, self.monthly_budget_cents);
            aggressive_debt_balance -= aggressive_payment;

            let aggressive_remaining_budget = self.monthly_budget_cents - aggressive_payment;
            aggressive_investments *= 1.0 + monthly_invest_rate;

            #[allow(clippy::cast_precision_loss)]
            let budget_f64 = aggressive_remaining_budget as f64;
            aggressive_investments += budget_f64;

            // Path 2: Invest Difference
            let invest_interest =
                (invest_debt_balance * i64::from(self.debt.interest_rate_pct)) / 100 / 12;
            invest_debt_balance += invest_interest;

            let invest_payment = std::cmp::min(invest_debt_balance, self.debt.min_payment_cents);
            let invest_payment_actual = std::cmp::min(invest_payment, self.monthly_budget_cents);
            invest_debt_balance -= invest_payment_actual;

            let invest_remaining_budget = self.monthly_budget_cents - invest_payment_actual;
            invest_investments *= 1.0 + monthly_invest_rate;

            #[allow(clippy::cast_precision_loss)]
            let inv_budget_f64 = invest_remaining_budget as f64;
            invest_investments += inv_budget_f64;
        }

        #[allow(clippy::cast_possible_truncation)]
        let aggressive_net_worth = aggressive_investments.round() as i64 - aggressive_debt_balance;

        #[allow(clippy::cast_possible_truncation)]
        let invest_net_worth = invest_investments.round() as i64 - invest_debt_balance;

        let optimal_strategy = match aggressive_net_worth.cmp(&invest_net_worth) {
            std::cmp::Ordering::Greater => OptimalStrategy::PayDebtAggressively,
            std::cmp::Ordering::Less => OptimalStrategy::InvestDifference,
            std::cmp::Ordering::Equal => OptimalStrategy::Neutral,
        };

        ArbitrageResult {
            aggressive_debt_net_worth_cents: aggressive_net_worth,
            invest_difference_net_worth_cents: invest_net_worth,
            optimal_strategy,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arbitrage_higher_investment_return() {
        // 3% debt, 7% investment return. Should invest.
        let debt = Debt {
            name: "Mortgage".to_string(),
            balance_cents: 1_000_000, // $10,000
            interest_rate_pct: 3,
            min_payment_cents: 10_000, // $100
        };

        let sim = ArbitrageSimulator::new(debt, 50_000, 7.0, 120); // 10 years, $500/mo budget
        let result = sim.simulate();

        assert_eq!(result.optimal_strategy, OptimalStrategy::InvestDifference);
        assert!(result.invest_difference_net_worth_cents > result.aggressive_debt_net_worth_cents);
    }

    #[test]
    fn test_arbitrage_higher_debt_interest() {
        // 10% debt, 5% investment return. Should pay debt aggressively.
        let debt = Debt {
            name: "Credit Card".to_string(),
            balance_cents: 1_000_000, // $10,000
            interest_rate_pct: 10,
            min_payment_cents: 10_000, // $100
        };

        let sim = ArbitrageSimulator::new(debt, 50_000, 5.0, 120); // 10 years, $500/mo budget
        let result = sim.simulate();

        assert_eq!(
            result.optimal_strategy,
            OptimalStrategy::PayDebtAggressively
        );
        assert!(result.aggressive_debt_net_worth_cents > result.invest_difference_net_worth_cents);
    }
}
