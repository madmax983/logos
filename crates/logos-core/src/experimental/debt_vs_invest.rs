#![cfg(feature = "nova")]

//! Debt vs Invest Analyzer
//!
//! A simulator that compares the mathematical outcome of putting extra money
//! toward paying off debt vs. investing it in the market.

use crate::experimental::debt_optimizer::Debt;

/// The outcome of a single strategy in the analysis.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StrategyOutcome {
    /// The number of months it took to pay off the debt.
    pub months_to_payoff: u32,
    /// Total amount of interest paid on the debt in cents.
    pub total_interest_paid_cents: i64,
    /// The final balance of the investment portfolio at the end of the simulation.
    pub final_investment_balance_cents: i64,
    /// The final net worth (investments - debt) at the end of the simulation.
    pub final_net_worth_cents: i64,
}

/// The result of comparing the "Payoff Debt" and "Invest" strategies.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebtVsInvestResult {
    /// Outcome if the extra cash is aggressively put toward the debt.
    pub aggressive_payoff_outcome: StrategyOutcome,
    /// Outcome if the extra cash is invested while paying only the minimum on the debt.
    pub aggressive_invest_outcome: StrategyOutcome,
    /// The net worth difference (Invest - Payoff) in cents at the end of the simulation horizon.
    /// A positive number means investing was better; negative means paying off debt was better.
    pub net_benefit_of_investing_cents: i64,
}

/// An analyzer that simulates capital allocation over a given time horizon.
#[derive(Debug, Clone)]
pub struct DebtVsInvestAnalyzer {
    debt: Debt,
    initial_investment_cents: i64,
    monthly_extra_cash_cents: i64,
    annual_investment_return_pct: f64,
    simulation_months: u32,
}

impl DebtVsInvestAnalyzer {
    /// Creates a new `DebtVsInvestAnalyzer`.
    ///
    /// # Arguments
    /// * `debt` - The debt to analyze.
    /// * `initial_investment_cents` - Starting investment portfolio balance.
    /// * `monthly_extra_cash_cents` - The extra cash available each month to either invest or pay extra on the debt.
    /// * `annual_investment_return_pct` - The expected annual return of the investments (e.g., 7.0).
    /// * `simulation_months` - The time horizon to compare the two strategies over.
    #[must_use]
    pub const fn new(
        debt: Debt,
        initial_investment_cents: i64,
        monthly_extra_cash_cents: i64,
        annual_investment_return_pct: f64,
        simulation_months: u32,
    ) -> Self {
        Self {
            debt,
            initial_investment_cents,
            monthly_extra_cash_cents,
            annual_investment_return_pct,
            simulation_months,
        }
    }

    /// Runs the simulation for both strategies and returns the comparison.
    #[must_use]
    pub fn analyze(&self) -> DebtVsInvestResult {
        let aggressive_payoff = self.simulate_strategy(true);
        let aggressive_invest = self.simulate_strategy(false);

        let net_benefit =
            aggressive_invest.final_net_worth_cents - aggressive_payoff.final_net_worth_cents;

        DebtVsInvestResult {
            aggressive_payoff_outcome: aggressive_payoff,
            aggressive_invest_outcome: aggressive_invest,
            net_benefit_of_investing_cents: net_benefit,
        }
    }

    fn simulate_strategy(&self, prioritize_debt: bool) -> StrategyOutcome {
        let mut debt_balance = self.debt.balance_cents;
        let mut investment_balance = self.initial_investment_cents;
        let mut total_interest_paid = 0;
        let mut months_to_payoff = 0;

        let monthly_debt_interest_rate = f64::from(self.debt.interest_rate_pct) / 100.0 / 12.0;
        let monthly_investment_return = self.annual_investment_return_pct / 100.0 / 12.0;

        for month in 1..=self.simulation_months {
            // 1. Calculate Debt Interest
            if debt_balance > 0 {
                #[allow(clippy::cast_precision_loss)]
                let interest_charge_f = debt_balance as f64 * monthly_debt_interest_rate;
                #[allow(clippy::cast_possible_truncation)]
                let interest_charge = interest_charge_f.round() as i64;
                debt_balance += interest_charge;
                total_interest_paid += interest_charge;
            }

            // 2. Calculate Investment Return
            #[allow(clippy::cast_precision_loss)]
            let inv_growth_f = investment_balance as f64 * monthly_investment_return;
            #[allow(clippy::cast_possible_truncation)]
            let inv_growth = inv_growth_f.round() as i64;
            investment_balance += inv_growth;

            // 3. Allocate Cash
            let mut cash_available = self.debt.min_payment_cents + self.monthly_extra_cash_cents;

            if debt_balance > 0 {
                let payment = if prioritize_debt {
                    // Put all available cash toward the debt
                    std::cmp::min(cash_available, debt_balance)
                } else {
                    // Pay minimum on debt, put the rest in investments
                    std::cmp::min(self.debt.min_payment_cents, debt_balance)
                };

                debt_balance -= payment;
                cash_available -= payment;

                if debt_balance == 0 && months_to_payoff == 0 {
                    months_to_payoff = month;
                }
            }

            // Any remaining cash goes to investments
            if cash_available > 0 {
                investment_balance += cash_available;
            }
        }

        // If debt was never paid off, record the total simulation months
        if months_to_payoff == 0 && debt_balance > 0 {
            months_to_payoff = self.simulation_months;
        }

        StrategyOutcome {
            months_to_payoff,
            total_interest_paid_cents: total_interest_paid,
            final_investment_balance_cents: investment_balance,
            final_net_worth_cents: investment_balance - debt_balance,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debt_vs_invest_high_interest_debt() {
        // High interest debt (15%) vs low market return (7%)
        let debt = Debt {
            name: "Credit Card".to_string(),
            balance_cents: 1_000_000, // $10,000
            interest_rate_pct: 15,
            min_payment_cents: 20_000, // $200 minimum
        };

        let analyzer = DebtVsInvestAnalyzer::new(
            debt, 500_000, // $5,000 initial investments
            80_000,  // $800 extra cash per month ($1000 total allocated)
            7.0,     // 7% expected return
            36,      // 3 years simulation
        );

        let result = analyzer.analyze();

        // With 15% debt and 7% investments, paying off the debt should be mathematically superior.
        // Therefore, the net benefit of investing should be negative.
        assert!(result.net_benefit_of_investing_cents < 0);

        // Aggressive payoff should result in less total interest paid
        assert!(
            result.aggressive_payoff_outcome.total_interest_paid_cents
                < result.aggressive_invest_outcome.total_interest_paid_cents
        );

        // Aggressive payoff should pay off the debt faster
        assert!(
            result.aggressive_payoff_outcome.months_to_payoff
                < result.aggressive_invest_outcome.months_to_payoff
        );
    }

    #[test]
    fn test_debt_vs_invest_low_interest_debt() {
        // Low interest debt (3%) vs high market return (10%)
        let debt = Debt {
            name: "Mortgage".to_string(),
            balance_cents: 10_000_000, // $100,000
            interest_rate_pct: 3,
            min_payment_cents: 100_000, // $1,000 minimum
        };

        let analyzer = DebtVsInvestAnalyzer::new(
            debt, 5_000_000, // $50,000 initial investments
            200_000,   // $2,000 extra cash per month
            10.0,      // 10% expected return
            60,        // 5 years simulation
        );

        let result = analyzer.analyze();

        // With 3% debt and 10% investments, investing the extra cash should be mathematically superior.
        // Therefore, the net benefit of investing should be positive.
        assert!(result.net_benefit_of_investing_cents > 0);
    }
}
