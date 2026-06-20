#![cfg(feature = "nova")]

//! Sabbatical Simulator
//!
//! Simulates the financial impact of taking a "mini-retirement" or sabbatical
//! on the journey to Financial Independence. It compares a baseline scenario
//! of continuous work against a scenario where you take a break, burning down
//! assets instead of saving, before returning to the workforce.

use crate::planning::fire::FireSimulator;

/// The result of a sabbatical simulation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SabbaticalResult {
    /// Months to reach FIRE target without taking a sabbatical.
    pub baseline_months: u32,
    /// Months to reach FIRE target if you take the sabbatical.
    pub sabbatical_scenario_months: u32,
    /// Total months the FIRE date is delayed.
    pub months_delayed: u32,
}

/// Simulates the impact of a sabbatical on financial independence.
#[derive(Debug, Clone)]
pub struct SabbaticalSimulator {
    fire_sim: FireSimulator,
    initial_net_worth_cents: i64,
    monthly_income_cents: i64,
    sabbatical_duration_months: u32,
    sabbatical_monthly_spend_cents: i64,
    annual_return_pct: f64,
}

impl SabbaticalSimulator {
    /// Creates a new `SabbaticalSimulator`.
    ///
    /// # Arguments
    /// * `fire_sim` - The base FIRE simulator defining working expenses and FIRE target.
    /// * `initial_net_worth_cents` - Current net worth.
    /// * `monthly_income_cents` - Current monthly income after taxes (when working).
    /// * `sabbatical_duration_months` - How long the sabbatical will last.
    /// * `sabbatical_monthly_spend_cents` - Monthly expenses during the sabbatical (e.g., travel).
    /// * `annual_return_pct` - Expected annual return on investments (e.g., 7.0).
    #[must_use]
    pub const fn new(
        fire_sim: FireSimulator,
        initial_net_worth_cents: i64,
        monthly_income_cents: i64,
        sabbatical_duration_months: u32,
        sabbatical_monthly_spend_cents: i64,
        annual_return_pct: f64,
    ) -> Self {
        Self {
            fire_sim,
            initial_net_worth_cents,
            monthly_income_cents,
            sabbatical_duration_months,
            sabbatical_monthly_spend_cents,
            annual_return_pct,
        }
    }

    /// Simulates the time to reach FIRE under a specific scenario.
    #[must_use]
    fn simulate(&self, take_sabbatical: bool) -> u32 {
        let mut net_worth = self.initial_net_worth_cents;
        let working_expenses = self.fire_sim.monthly_expenses_cents();
        let target = self.fire_sim.fire_number_cents();

        if target == 0 {
            return 0; // Already FIRE
        }

        let monthly_return_rate = if self.annual_return_pct > 0.0 {
            (1.0 + self.annual_return_pct / 100.0).powf(1.0 / 12.0) - 1.0
        } else {
            0.0
        };

        // Check for impossibility: if we never grow and expenses >= income
        if working_expenses >= self.monthly_income_cents && self.annual_return_pct <= 0.0 {
            return u32::MAX;
        }

        let mut months = 0;

        while months < 1200 {
            // Max 100 years
            if net_worth >= target {
                return months;
            }

            // Apply investment returns
            if net_worth > 0 {
                #[allow(clippy::cast_precision_loss)]
                let returns = (net_worth as f64) * monthly_return_rate;
                #[allow(clippy::cast_possible_truncation)]
                let returns_cents = returns.round() as i64;
                net_worth += returns_cents;
            }

            let is_on_sabbatical = take_sabbatical && months < self.sabbatical_duration_months;

            if is_on_sabbatical {
                // Burning money
                net_worth -= self.sabbatical_monthly_spend_cents;
            } else {
                // Working
                let savings = self.monthly_income_cents - working_expenses;
                net_worth += savings;
            }

            months += 1;
        }

        u32::MAX
    }

    /// Calculates the impact of the sabbatical compared to a baseline of continuous work.
    #[must_use]
    pub fn calculate(&self) -> SabbaticalResult {
        let baseline_months = self.simulate(false);
        let sabbatical_scenario_months = self.simulate(true);

        let months_delayed = if sabbatical_scenario_months > baseline_months
            && sabbatical_scenario_months < u32::MAX
        {
            sabbatical_scenario_months - baseline_months
        } else if sabbatical_scenario_months == u32::MAX {
            u32::MAX
        } else {
            0
        };

        SabbaticalResult {
            baseline_months,
            sabbatical_scenario_months,
            months_delayed,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sabbatical_impact() {
        let fire_sim = FireSimulator::new(400_000); // $4k expenses -> $1.2M target

        let sim = SabbaticalSimulator::new(
            fire_sim, 20_000_000, // $200k starting (in cents: 200,000 * 100 = 20,000,000)
            800_000,    // $8k income -> $4k savings/mo
            12,         // 1 year sabbatical
            500_000,    // $5k spend during sabbatical
            7.0,        // 7% return
        );

        let result = sim.calculate();

        assert!(result.baseline_months > 0);
        assert!(result.sabbatical_scenario_months > result.baseline_months);

        // Sabbatical should delay FIRE by more than just the 12 months,
        // because we burned $60k AND missed out on $48k of savings.
        assert!(result.months_delayed > 12);
    }

    #[test]
    fn test_no_sabbatical_has_zero_delay() {
        let fire_sim = FireSimulator::new(400_000);

        let sim = SabbaticalSimulator::new(
            fire_sim, 20_000_000, 800_000, 0, // 0 months sabbatical
            500_000, 7.0,
        );

        let result = sim.calculate();
        assert_eq!(result.baseline_months, result.sabbatical_scenario_months);
        assert_eq!(result.months_delayed, 0);
    }

    #[test]
    fn test_already_fire() {
        let fire_sim = FireSimulator::new(400_000); // target 1.2M

        let sim = SabbaticalSimulator::new(
            fire_sim,
            150_000_000, // 1.5M starting, already FIRE
            800_000,
            12,
            500_000,
            7.0,
        );

        let result = sim.calculate();
        assert_eq!(result.baseline_months, 0);
        assert_eq!(result.sabbatical_scenario_months, 0);
    }
}
