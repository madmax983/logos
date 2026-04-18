//! Lifestyle Creep Impact Simulator
//!
//! # The Silent Wealth Killer
//!
//! Lifestyle creep (or lifestyle inflation) happens when discretionary income
//! increases and expenses rise proportionately. This module simulates how
//! different levels of lifestyle creep delay Financial Independence (FIRE).
//! By modeling the "creep factor" (the percentage of a raise that goes toward
//! new lifestyle expenses instead of savings), users can visualize the true
//! time cost of upgrading their lifestyle.

/// The result of simulating lifestyle creep over time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreepSimulationResult {
    /// Total months required to reach FIRE under this scenario.
    pub months_to_fire: u32,
    /// The final FIRE target required at the time of retirement, in cents.
    pub final_fire_number_cents: i64,
    /// The final monthly expenses at the time of retirement, in cents.
    pub final_monthly_expenses_cents: i64,
}

/// A simulator to analyze the impact of lifestyle inflation on the journey to FIRE.
#[derive(Debug, Clone)]
pub struct LifestyleCreepSimulator {
    /// Starting monthly net income in cents.
    pub starting_monthly_income_cents: i64,
    /// Starting monthly expenses in cents.
    pub starting_monthly_expenses_cents: i64,
    /// Expected annual income growth rate (e.g., 0.05 for 5%).
    pub annual_income_growth_rate: f64,
    /// Expected annualized investment return rate (e.g., 0.07 for 7%).
    pub annual_investment_return_rate: f64,
    /// Safe withdrawal rate for FIRE calculation (e.g., 0.04 for 4%).
    pub safe_withdrawal_rate: f64,
}

impl LifestyleCreepSimulator {
    /// Creates a new `LifestyleCreepSimulator`.
    ///
    /// # Arguments
    /// * `starting_monthly_income_cents` - The initial monthly take-home pay.
    /// * `starting_monthly_expenses_cents` - The initial monthly expenses.
    /// * `annual_income_growth_rate` - Expected yearly raise (e.g. `0.05` for 5%).
    /// * `annual_investment_return_rate` - Expected real investment return (e.g. `0.07` for 7%).
    /// * `safe_withdrawal_rate` - Withdrawal rate to determine the FIRE number (e.g. `0.04` for 4%).
    #[must_use]
    pub const fn new(
        starting_monthly_income_cents: i64,
        starting_monthly_expenses_cents: i64,
        annual_income_growth_rate: f64,
        annual_investment_return_rate: f64,
        safe_withdrawal_rate: f64,
    ) -> Self {
        Self {
            starting_monthly_income_cents,
            starting_monthly_expenses_cents,
            annual_income_growth_rate,
            annual_investment_return_rate,
            safe_withdrawal_rate,
        }
    }

    /// Simulates the journey to FIRE given a specific "creep factor".
    ///
    /// # Arguments
    /// * `creep_factor` - The percentage of an income raise that is absorbed by new
    ///   expenses. For example, `0.5` means 50% of the raise goes to lifestyle inflation,
    ///   while the other 50% goes to additional savings. A value of `0.0` represents
    ///   strict lifestyle maintenance, and `1.0` means every new dollar earned is spent.
    #[must_use]
    pub fn simulate(&self, creep_factor: f64) -> CreepSimulationResult {
        let mut months = 0;

        #[allow(clippy::cast_precision_loss)]
        let mut current_monthly_income = self.starting_monthly_income_cents as f64;

        #[allow(clippy::cast_precision_loss)]
        let mut current_monthly_expenses = self.starting_monthly_expenses_cents as f64;

        let mut portfolio_cents = 0.0;
        let monthly_return_rate = self.annual_investment_return_rate / 12.0;

        loop {
            // Check if we hit FIRE at current expense level
            let fire_number = (current_monthly_expenses * 12.0) / self.safe_withdrawal_rate;
            if portfolio_cents >= fire_number {
                break;
            }

            // End of month: add savings to portfolio and apply growth
            let savings = current_monthly_income - current_monthly_expenses;
            portfolio_cents += savings;

            if portfolio_cents > 0.0 {
                portfolio_cents *= 1.0 + monthly_return_rate;
            }

            months += 1;

            // Apply annual raises and lifestyle creep
            if months > 0 && months % 12 == 0 {
                let new_income = current_monthly_income * (1.0 + self.annual_income_growth_rate);
                let income_increase = new_income - current_monthly_income;

                // Creep factor dictates how much of the raise is spent
                let expense_increase = income_increase * creep_factor;

                current_monthly_income = new_income;
                current_monthly_expenses += expense_increase;
            }

            // Cap to avoid infinite loops in extreme inputs (e.g. 100 years max)
            if months >= 1200 {
                break;
            }
        }

        #[allow(clippy::cast_possible_truncation)]
        let final_fire_number_cents =
            ((current_monthly_expenses * 12.0) / self.safe_withdrawal_rate).round() as i64;

        #[allow(clippy::cast_possible_truncation)]
        let final_monthly_expenses_cents = current_monthly_expenses.round() as i64;

        CreepSimulationResult {
            months_to_fire: months,
            final_fire_number_cents,
            final_monthly_expenses_cents,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_creep() {
        let sim = LifestyleCreepSimulator::new(
            1_000_000, // $10,000 income
            500_000,   // $5,000 expenses
            0.05,      // 5% annual raise
            0.07,      // 7% return
            0.04,      // 4% SWR
        );

        let result = sim.simulate(0.0);

        // Without creep, expenses remain $5,000/mo.
        // FIRE number is $5,000 * 12 / 0.04 = $1,500,000 = 150_000_000 cents.
        assert_eq!(result.final_monthly_expenses_cents, 500_000);
        assert_eq!(result.final_fire_number_cents, 150_000_000);

        // Months to FIRE should be calculated.
        assert!(result.months_to_fire > 0);
        assert!(result.months_to_fire < 1200);
    }

    #[test]
    fn test_high_creep_delays_fire() {
        let sim = LifestyleCreepSimulator::new(1_000_000, 500_000, 0.05, 0.07, 0.04);

        let result_zero_creep = sim.simulate(0.0);
        let result_half_creep = sim.simulate(0.5);
        let result_full_creep = sim.simulate(1.0);

        // More creep should delay FIRE.
        assert!(result_half_creep.months_to_fire > result_zero_creep.months_to_fire);
        assert!(result_full_creep.months_to_fire > result_half_creep.months_to_fire);

        // More creep should increase final expenses and the required FIRE number.
        assert!(result_half_creep.final_monthly_expenses_cents > 500_000);
        assert!(result_half_creep.final_fire_number_cents > 150_000_000);
    }

    #[test]
    fn test_impossible_fire() {
        let sim = LifestyleCreepSimulator::new(
            500_000, // $5,000 income
            500_000, // $5,000 expenses (saving $0)
            0.02,    // 2% raise
            0.07, 0.04,
        );

        // 100% creep means they spend all raises and never save.
        let result = sim.simulate(1.0);

        // Should hit the 100-year cap (1200 months)
        assert_eq!(result.months_to_fire, 1200);
    }
}
