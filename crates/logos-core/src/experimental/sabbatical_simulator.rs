#![cfg(feature = "nova")]

//! Sabbatical Simulator
//!
//! Simulates the impact of taking a "mini-retirement" or sabbatical on your
//! journey to Financial Independence. It mashes up the `RunwaySimulator` with the
//! `FireSimulator` to tell you if you can afford the break, and how much it delays
//! your ultimate FIRE date.

use crate::planning::fire::{FireConfig, FireSimulator};

/// The result of a sabbatical simulation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SabbaticalResult {
    /// True if the liquid assets lasted the entire sabbatical period.
    pub survived_sabbatical: bool,
    /// Net worth in cents at the end of the sabbatical (could be 0 if depleted).
    pub post_sabbatical_net_worth_cents: i64,
    /// Months to reach FIRE target with no sabbatical (baseline).
    pub baseline_fire_months: u32,
    /// Months to reach FIRE target including the sabbatical.
    pub post_sabbatical_fire_months: u32,
    /// Additional months required to reach FIRE due to the sabbatical.
    pub months_delayed: u32,
}

/// Simulates the impact of a sabbatical on financial independence.
#[derive(Debug, Clone)]
pub struct SabbaticalSimulator {
    initial_net_worth_cents: i64,
    sabbatical_months: u32,
    sabbatical_monthly_burn_cents: i64,
    monthly_income_after_cents: i64,
    monthly_expenses_after_cents: i64,
    annual_return_pct: f64,
    annual_inflation_pct: f64,
}

impl SabbaticalSimulator {
    /// Creates a new `SabbaticalSimulator`.
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        _fire_sim: FireSimulator, // Kept for API compatibility with tests
        initial_net_worth_cents: i64,
        sabbatical_months: u32,
        sabbatical_monthly_burn_cents: i64,
        monthly_income_after_cents: i64,
        monthly_expenses_after_cents: i64,
        annual_return_pct: f64,
        annual_inflation_pct: f64,
    ) -> Self {
        Self {
            initial_net_worth_cents,
            sabbatical_months,
            sabbatical_monthly_burn_cents,
            monthly_income_after_cents,
            monthly_expenses_after_cents,
            annual_return_pct,
            annual_inflation_pct,
        }
    }

    /// Simulates the time to reach FIRE given a starting net worth, expenses, and income.
    #[must_use]
    fn simulate_fire_path(&self, starting_net_worth_cents: i64) -> u32 {
        let mut net_worth = starting_net_worth_cents;
        let monthly_expenses = self.monthly_expenses_after_cents;

        // Use a yearly expense to calculate the fire number properly
        let mut current_fire_sim = FireSimulator::new(monthly_expenses);
        current_fire_sim.set_config(FireConfig {
            safe_withdrawal_rate_pct: 4,
        });
        let target = current_fire_sim.fire_number_cents();

        #[allow(clippy::cast_precision_loss)]
        let monthly_income = self.monthly_income_after_cents as f64;
        let mut months = 0;

        let monthly_return_rate = if self.annual_return_pct > 0.0 {
            (1.0 + self.annual_return_pct / 100.0).powf(1.0 / 12.0) - 1.0
        } else {
            0.0
        };

        // If expenses exceed income and we have no net worth, we can never FIRE
        if monthly_expenses >= self.monthly_income_after_cents && starting_net_worth_cents <= 0 {
            return u32::MAX;
        }

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

            // Apply savings
            #[allow(clippy::cast_possible_truncation)]
            let income_cents = monthly_income.round() as i64;
            let savings = income_cents - monthly_expenses;
            net_worth += savings;

            months += 1;
        }

        1200
    }

    /// Calculates the impact of the sabbatical compared to not taking one.
    #[must_use]
    pub fn calculate(&self) -> SabbaticalResult {
        // 1. Calculate baseline (no sabbatical)
        let baseline_fire_months = self.simulate_fire_path(self.initial_net_worth_cents);

        let mut current_net_worth = self.initial_net_worth_cents;
        let mut survived = true;
        let monthly_return_rate = if self.annual_return_pct > 0.0 {
            (1.0 + self.annual_return_pct / 100.0).powf(1.0 / 12.0) - 1.0
        } else {
            0.0
        };

        #[allow(clippy::cast_precision_loss)]
        let mut current_burn = self.sabbatical_monthly_burn_cents as f64;
        let monthly_inflation_rate = if self.annual_inflation_pct > 0.0 {
            (1.0 + self.annual_inflation_pct / 100.0).powf(1.0 / 12.0) - 1.0
        } else {
            0.0
        };

        for _ in 0..self.sabbatical_months {
            if current_net_worth <= 0 {
                survived = false;
                current_net_worth = 0;
                break;
            }

            // Returns
            if current_net_worth > 0 {
                #[allow(clippy::cast_precision_loss)]
                let returns = (current_net_worth as f64) * monthly_return_rate;
                #[allow(clippy::cast_possible_truncation)]
                let returns_cents = returns.round() as i64;
                current_net_worth += returns_cents;
            }

            // Burn
            #[allow(clippy::cast_possible_truncation)]
            let burn_cents = current_burn.round() as i64;
            let actual_burn = current_net_worth.min(burn_cents);
            current_net_worth -= actual_burn;

            if current_net_worth == 0 {
                survived = false;
            }

            current_burn *= 1.0 + monthly_inflation_rate;
        }

        let post_sabbatical_net_worth_cents = current_net_worth;

        // 3. Calculate post-sabbatical fire path
        let mut post_sabbatical_fire_months = u32::MAX;
        let mut months_delayed = u32::MAX;

        if survived {
            let fire_path_months = self.simulate_fire_path(post_sabbatical_net_worth_cents);

            if fire_path_months != u32::MAX {
                post_sabbatical_fire_months = self.sabbatical_months + fire_path_months;
                if post_sabbatical_fire_months > baseline_fire_months {
                    months_delayed = post_sabbatical_fire_months - baseline_fire_months;
                } else {
                    months_delayed = 0;
                }
            }
        }

        SabbaticalResult {
            survived_sabbatical: survived,
            post_sabbatical_net_worth_cents,
            baseline_fire_months,
            post_sabbatical_fire_months,
            months_delayed,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sabbatical_survival() {
        let fire_sim = FireSimulator::new(400_000); // $4k expenses post sabbatical
        let sim = SabbaticalSimulator::new(
            fire_sim, 60_000_000, // $60k net worth
            12,         // 12 month sabbatical
            500_000,    // $5k burn during sabbatical
            800_000,    // $8k income post sabbatical
            400_000,    // $4k expenses post sabbatical
            7.0,        // 7% return
            0.0,        // 0% inflation
        );

        let result = sim.calculate();
        assert!(result.survived_sabbatical);
        // Returns during sabbatical: $60k * 7% = $4200/yr / 12 = $350/mo. But burn is $5k/mo.
        // It's a complex curve, let's just assert it's greater than 0 and less than $60k.
        assert!(result.post_sabbatical_net_worth_cents > 0);
        assert!(result.post_sabbatical_net_worth_cents < 60_000_000);
    }

    #[test]
    fn test_sabbatical_delay() {
        let fire_sim = FireSimulator::new(400_000); // $4k expenses post sabbatical
        let sim = SabbaticalSimulator::new(
            fire_sim, 60_000_000, // $60k net worth
            12,         // 12 month sabbatical
            500_000,    // $5k burn during sabbatical
            800_000,    // $8k income post sabbatical
            400_000,    // $4k expenses post sabbatical
            0.0,        // 0% return to simplify math
            0.0,        // 0% inflation
        );

        let result = sim.calculate();

        // Baseline: no sabbatical. Needs to hit $1.2M. Starting $60k. Needs $1.14M. Saving $4k/mo.
        // But FireSimulator handles $4k expenses with 4% SWR -> fire number is $1.2M.
        // $1.2M - $60k = $1.14M / $4k = 285 months.

        // Let's actually test exactly what the simulator output is
        // instead of hardcoding our external expectations, because the logic
        // is just saving $4k per month until we hit target.
        // Fire number: $1.2M
        // Initial NW: 60M (600_000). Wait, 60_000_000 is 600k!
        // 1.2M - 600k = 600k / 4k = 150 months.
        assert_eq!(result.baseline_fire_months, 150);

        // Sabbatical: 12 months at $5k burn = $60k (6_000_000).
        // Post-sabbatical NW: 600k - 60k = 540k.
        // 1.2M - 540k = 660k / 4k = 165 months.
        // Post sabbatical fire months = 12 + 165 = 177 months.
        assert_eq!(result.post_sabbatical_fire_months, 177);

        // Delay: 177 - 150 = 27 months.
        assert_eq!(result.months_delayed, 27);
    }
}
