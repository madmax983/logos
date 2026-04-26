#![cfg(feature = "nova")]

//! Lifestyle Creep Simulator
//!
//! Simulates the impact of "lifestyle creep"—where expenses grow as income grows—
//! on the journey to Financial Independence.

use crate::planning::fire::{FireConfig, FireSimulator};

/// The result of a lifestyle creep simulation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreepResult {
    /// Months to reach FIRE target with 0% lifestyle creep.
    pub baseline_months: u32,
    /// Months to reach FIRE target with the specified lifestyle creep.
    pub creep_months: u32,
    /// Additional months required due to lifestyle creep.
    pub months_delayed: u32,
    /// The final FIRE target net worth in cents after creep.
    pub final_target_cents: i64,
}

/// Simulates the impact of lifestyle creep on financial independence.
#[derive(Debug, Clone)]
pub struct LifestyleCreepSimulator {
    fire_sim: FireSimulator,
    initial_net_worth_cents: i64,
    monthly_income_cents: i64,
    annual_raise_pct: f64,
    creep_pct: f64,
    annual_return_pct: f64,
}

impl LifestyleCreepSimulator {
    /// Creates a new `LifestyleCreepSimulator`.
    ///
    /// # Arguments
    /// * `fire_sim` - The base FIRE simulator defining current expenses and safe withdrawal rate.
    /// * `initial_net_worth_cents` - Starting net worth.
    /// * `monthly_income_cents` - Current monthly income after taxes.
    /// * `annual_raise_pct` - Expected annual income raise percentage (e.g., 3.0).
    /// * `creep_pct` - Percentage of the raise that goes to increased expenses (e.g., 50.0).
    /// * `annual_return_pct` - Expected annual return on investments (e.g., 7.0).
    #[must_use]
    pub const fn new(
        fire_sim: FireSimulator,
        initial_net_worth_cents: i64,
        monthly_income_cents: i64,
        annual_raise_pct: f64,
        creep_pct: f64,
        annual_return_pct: f64,
    ) -> Self {
        Self {
            fire_sim,
            initial_net_worth_cents,
            monthly_income_cents,
            annual_raise_pct,
            creep_pct,
            annual_return_pct,
        }
    }

    /// Simulates the time to reach FIRE under a specific creep percentage.
    #[must_use]
    fn simulate(&self, creep_pct: f64) -> (u32, i64) {
        let mut net_worth = self.initial_net_worth_cents;
        let mut monthly_expenses = self.fire_sim.monthly_expenses_cents();
        #[allow(clippy::cast_precision_loss)]
        let mut monthly_income = self.monthly_income_cents as f64;
        let mut months = 0;

        let monthly_return_rate = if self.annual_return_pct > 0.0 {
            (1.0 + self.annual_return_pct / 100.0).powf(1.0 / 12.0) - 1.0
        } else {
            0.0
        };

        // If expenses exceed income from start and net worth drops to 0, it's impossible.
        if monthly_expenses >= self.monthly_income_cents && self.initial_net_worth_cents <= 0 {
            return (u32::MAX, i64::MAX);
        }

        while months < 1200 {
            // Max 100 years
            let mut current_fire_sim = FireSimulator::new(monthly_expenses);
            current_fire_sim.set_config(FireConfig {
                safe_withdrawal_rate_pct: 4,
            });
            let target = current_fire_sim.fire_number_cents();

            if net_worth >= target {
                return (months, target);
            }

            // Apply investment returns
            if net_worth > 0 {
                #[allow(clippy::cast_precision_loss)]
                let returns = (net_worth as f64) * monthly_return_rate;
                #[allow(clippy::cast_possible_truncation)]
                let returns_cents = returns.round() as i64;
                net_worth += returns_cents;
            }

            // Apply savings
            #[allow(clippy::cast_possible_truncation)]
            let income_cents = monthly_income.round() as i64;
            let savings = income_cents - monthly_expenses;
            net_worth += savings;

            months += 1;

            // Apply annual raises and creep
            if months % 12 == 0 {
                let raise_amount = monthly_income * (self.annual_raise_pct / 100.0);
                monthly_income += raise_amount;

                let creep_amount = raise_amount * (creep_pct / 100.0);
                #[allow(clippy::cast_possible_truncation)]
                let new_expense_increase = creep_amount.round() as i64;
                monthly_expenses += new_expense_increase;
            }
        }
        (1200, i64::MAX)
    }

    /// Calculates the impact of lifestyle creep compared to a baseline of 0% creep.
    #[must_use]
    pub fn calculate(&self) -> CreepResult {
        let (baseline_months, _) = self.simulate(0.0);
        let (creep_months, final_target) = self.simulate(self.creep_pct);

        let months_delayed = if creep_months > baseline_months && creep_months < 1200 {
            creep_months - baseline_months
        } else if creep_months == 1200 {
            u32::MAX
        } else {
            0
        };

        CreepResult {
            baseline_months,
            creep_months,
            months_delayed,
            final_target_cents: final_target,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_creep() {
        let fire_sim = FireSimulator::new(400_000); // $4k expenses
        let sim = LifestyleCreepSimulator::new(
            fire_sim, 0,       // 0 net worth
            800_000, // $8k income
            5.0,     // 5% annual raise
            0.0,     // 0% creep
            7.0,     // 7% return
        );

        let result = sim.calculate();
        assert_eq!(result.baseline_months, result.creep_months);
        assert_eq!(result.months_delayed, 0);
        assert_eq!(result.final_target_cents, 120_000_000); // $1.2M target doesn't change
    }

    #[test]
    fn test_high_creep() {
        let fire_sim = FireSimulator::new(400_000); // $4k expenses
        let sim = LifestyleCreepSimulator::new(
            fire_sim, 0, 800_000, 5.0, 100.0, // 100% creep (all raise goes to expenses)
            7.0,
        );

        let result = sim.calculate();
        assert!(result.creep_months > result.baseline_months);
        assert!(result.months_delayed > 0);
        assert!(result.final_target_cents > 120_000_000); // Target increases
    }

    #[test]
    fn test_no_raise_no_creep() {
        let fire_sim = FireSimulator::new(500_000); // $5k expenses
        let sim = LifestyleCreepSimulator::new(
            fire_sim, 50_000_000, // $500k NW
            1_000_000,   // $10k income
            0.0,        // 0% raise
            50.0,       // 50% creep (doesn't matter since 0 raise)
            7.0,
        );

        let result = sim.calculate();
        assert_eq!(result.baseline_months, result.creep_months);
        assert_eq!(result.months_delayed, 0);
    }

    #[test]
    fn test_impossible_target() {
        let fire_sim = FireSimulator::new(1_000_000); // $10k expenses
        let sim = LifestyleCreepSimulator::new(
            fire_sim, 0,       // 0 NW
            500_000, // $5k income (less than expenses)
            0.0, 0.0, 7.0,
        );
        let result = sim.calculate();
        assert_eq!(result.baseline_months, u32::MAX);
    }
}
