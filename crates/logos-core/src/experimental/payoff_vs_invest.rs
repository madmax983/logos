//! Payoff vs Invest Simulator
//!
//! A simulator that answers the age-old question: "Should I pay off my debt early,
//! or invest the extra cash instead?"
//!
//! It simulates two parallel universes side-by-side:
//! 1. **Aggressive Payoff:** Any monthly surplus goes entirely towards debt (using the Avalanche method) until paid off, then is invested.
//! 2. **Minimums + Invest:** Only minimum payments are made on the debt, and all monthly surplus is invested in the market immediately.
//!
//! By comparing the total net worth of both universes after a set period, you can see
//! which strategy mathematically wins given your specific interest rates and expected market returns.

use crate::experimental::debt_optimizer::Debt;

/// Represents the net worth at the end of the simulation period for a specific strategy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UniverseResult {
    /// The total liquid assets at the end of the period.
    pub assets_cents: i64,
    /// The total remaining debt at the end of the period.
    pub remaining_debt_cents: i64,
    /// Net worth (assets - debt) at the end of the period.
    pub net_worth_cents: i64,
}

/// The result of simulating both the "Payoff" and "Invest" universes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PayoffVsInvestResult {
    /// The outcome if surplus was used to aggressively pay down debt first.
    pub aggressive_payoff_universe: UniverseResult,
    /// The outcome if surplus was invested while paying minimums on debt.
    pub minimums_and_invest_universe: UniverseResult,
    /// How much better (or worse) the aggressive payoff strategy was compared to investing.
    /// Positive means paying off debt was better; negative means investing was better.
    pub payoff_advantage_cents: i64,
}

/// A simulator that compares paying down debt vs investing extra cash.
#[derive(Debug, Clone)]
pub struct PayoffVsInvestSimulator {
    debts: Vec<Debt>,
    initial_assets_cents: i64,
    monthly_surplus_cents: i64, // The extra cash available AFTER paying minimums
    annual_market_return_pct: f64,
    simulation_months: u16,
}

impl PayoffVsInvestSimulator {
    /// Creates a new simulator.
    ///
    /// # Arguments
    /// * `initial_assets_cents` - Starting invested assets.
    /// * `monthly_surplus_cents` - Extra cash available each month *on top* of minimum debt payments.
    /// * `annual_market_return_pct` - Expected annual return of investments (e.g., 7.0 for 7%).
    /// * `simulation_months` - How many months to simulate (e.g., 120 for 10 years).
    #[must_use]
    pub const fn new(
        initial_assets_cents: i64,
        monthly_surplus_cents: i64,
        annual_market_return_pct: f64,
        simulation_months: u16,
    ) -> Self {
        Self {
            debts: Vec::new(),
            initial_assets_cents,
            monthly_surplus_cents,
            annual_market_return_pct,
            simulation_months,
        }
    }

    /// Adds a debt obligation to both simulation universes.
    pub fn add_debt(&mut self, debt: Debt) {
        self.debts.push(debt);
    }

    fn calculate_monthly_investment_growth(assets_cents: i64, annual_return_pct: f64) -> i64 {
        if assets_cents <= 0 || annual_return_pct <= 0.0 {
            return 0;
        }
        let monthly_return = annual_return_pct / 1200.0; // 7.0% -> 0.07 / 12
        #[allow(clippy::cast_precision_loss)]
        let assets_f64 = assets_cents as f64;
        let growth = assets_f64 * monthly_return;
        #[allow(clippy::cast_possible_truncation)]
        let growth_cents = growth.round() as i64;
        growth_cents
    }

    /// Runs the simulation and returns the comparison.
    #[must_use]
    pub fn simulate(&self) -> PayoffVsInvestResult {
        // --- Universe 1: Aggressive Payoff ---
        let mut u1_debts = self.debts.clone();
        let mut u1_assets = self.initial_assets_cents;

        for _ in 0..self.simulation_months {
            // Grow existing investments
            u1_assets +=
                Self::calculate_monthly_investment_growth(u1_assets, self.annual_market_return_pct);

            let mut remaining_surplus = self.monthly_surplus_cents;

            // Apply interest and pay minimums
            for debt in &mut u1_debts {
                if debt.balance_cents > 0 {
                    let interest =
                        (debt.balance_cents * i64::from(debt.interest_rate_pct)) / 100 / 12;
                    debt.balance_cents += interest;

                    let min_pay = std::cmp::min(debt.balance_cents, debt.min_payment_cents);
                    debt.balance_cents -= min_pay;

                    // If the debt is fully paid off, the minimum payment we would have made
                    // is now freed up to become part of our surplus for investing!
                    let freed_cashflow = debt.min_payment_cents - min_pay;
                    if freed_cashflow > 0 {
                        remaining_surplus += freed_cashflow;
                    }
                } else {
                    // Debt was already 0 before this month started, so we get to keep the entire min payment.
                    remaining_surplus += debt.min_payment_cents;
                }
            }

            // Avalanche: pay highest interest first
            u1_debts.sort_by(|a, b| b.interest_rate_pct.cmp(&a.interest_rate_pct));

            for debt in &mut u1_debts {
                if debt.balance_cents > 0 && remaining_surplus > 0 {
                    let pay = std::cmp::min(debt.balance_cents, remaining_surplus);
                    debt.balance_cents -= pay;
                    remaining_surplus -= pay;
                }
            }

            // If debt is gone, invest the rest of the surplus
            if remaining_surplus > 0 {
                u1_assets += remaining_surplus;
            }
        }

        let u1_debt_total: i64 = u1_debts.iter().map(|d| d.balance_cents).sum();
        let u1_net_worth = u1_assets - u1_debt_total;

        let universe_1 = UniverseResult {
            assets_cents: u1_assets,
            remaining_debt_cents: u1_debt_total,
            net_worth_cents: u1_net_worth,
        };

        // --- Universe 2: Minimums + Invest ---
        let mut u2_debts = self.debts.clone();
        let mut u2_assets = self.initial_assets_cents;

        for _ in 0..self.simulation_months {
            // Grow existing investments
            u2_assets +=
                Self::calculate_monthly_investment_growth(u2_assets, self.annual_market_return_pct);

            let mut u2_freed_cashflow = 0;

            // Apply interest and pay minimums
            for debt in &mut u2_debts {
                if debt.balance_cents > 0 {
                    let interest =
                        (debt.balance_cents * i64::from(debt.interest_rate_pct)) / 100 / 12;
                    debt.balance_cents += interest;

                    let min_pay = std::cmp::min(debt.balance_cents, debt.min_payment_cents);
                    debt.balance_cents -= min_pay;

                    let freed_cashflow = debt.min_payment_cents - min_pay;
                    if freed_cashflow > 0 {
                        u2_freed_cashflow += freed_cashflow;
                    }
                } else {
                    u2_freed_cashflow += debt.min_payment_cents;
                }
            }

            // Invest all surplus + any freed cashflow from fully paid off debts
            u2_assets += self.monthly_surplus_cents + u2_freed_cashflow;
        }

        let u2_debt_total: i64 = u2_debts.iter().map(|d| d.balance_cents).sum();
        let u2_net_worth = u2_assets - u2_debt_total;

        let universe_2 = UniverseResult {
            assets_cents: u2_assets,
            remaining_debt_cents: u2_debt_total,
            net_worth_cents: u2_net_worth,
        };

        PayoffVsInvestResult {
            aggressive_payoff_universe: universe_1,
            minimums_and_invest_universe: universe_2,
            payoff_advantage_cents: u1_net_worth - u2_net_worth,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_high_interest_debt_favors_payoff() {
        // High interest debt (20%) vs moderate market return (7%)
        let mut sim = PayoffVsInvestSimulator::new(
            0, 100_000, // $1,000 extra per month
            7.0,     // 7% return
            60,      // 5 years
        );

        sim.add_debt(Debt {
            name: "Credit Card".to_string(),
            balance_cents: 2_000_000,  // $20,000
            interest_rate_pct: 20,     // 20% interest!
            min_payment_cents: 40_000, // $400 min
        });

        let result = sim.simulate();

        // Paying off a 20% debt should definitely beat a 7% market return
        assert!(result.payoff_advantage_cents > 0);
        assert!(
            result.aggressive_payoff_universe.net_worth_cents
                > result.minimums_and_invest_universe.net_worth_cents
        );
    }

    #[test]
    fn test_low_interest_debt_favors_investing() {
        // Low interest debt (3%) vs moderate market return (7%)
        let mut sim = PayoffVsInvestSimulator::new(
            0, 100_000, // $1,000 extra per month
            7.0,     // 7% return
            120,     // 10 years
        );

        sim.add_debt(Debt {
            name: "Mortgage".to_string(),
            balance_cents: 30_000_000,  // $300,000
            interest_rate_pct: 3,       // 3% interest
            min_payment_cents: 150_000, // $1500 min
        });

        let result = sim.simulate();

        // Investing at 7% should beat paying down a 3% loan
        assert!(result.payoff_advantage_cents < 0);
        assert!(
            result.aggressive_payoff_universe.net_worth_cents
                < result.minimums_and_invest_universe.net_worth_cents
        );
    }

    #[test]
    fn test_zero_market_return() {
        let mut sim = PayoffVsInvestSimulator::new(100_000, 10_000, 0.0, 12);
        sim.add_debt(Debt {
            name: "Loan".to_string(),
            balance_cents: 50_000,
            interest_rate_pct: 5,
            min_payment_cents: 5_000,
        });

        let result = sim.simulate();
        assert!(result.payoff_advantage_cents > 0); // Paying down debt avoids interest, investing makes 0.
    }
}
