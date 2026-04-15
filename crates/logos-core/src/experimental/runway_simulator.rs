//! Financial Runway Simulator
//!
//! Calculates how long liquid assets will last given a monthly burn rate
//! and an annual inflation rate.

/// Represents the simulation result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunwayResult {
    /// The number of months the assets will last (capped at 1200, i.e., 100 years).
    pub months: u32,
    /// Total amount burned in cents over the runway period.
    pub total_burned_cents: i64,
}

/// A simulator for calculating financial runway.
#[derive(Debug, Clone)]
pub struct RunwaySimulator {
    liquid_assets_cents: i64,
    monthly_burn_cents: i64,
    annual_inflation_pct: f64,
}

impl RunwaySimulator {
    /// Creates a new `RunwaySimulator`.
    ///
    /// # Arguments
    /// * `liquid_assets_cents` - Total available liquid assets in cents.
    /// * `monthly_burn_cents` - Monthly expenses in cents.
    /// * `annual_inflation_pct` - Expected annual inflation rate (e.g., 3.0 for 3%).
    #[must_use]
    pub const fn new(
        liquid_assets_cents: i64,
        monthly_burn_cents: i64,
        annual_inflation_pct: f64,
    ) -> Self {
        Self {
            liquid_assets_cents,
            monthly_burn_cents,
            annual_inflation_pct,
        }
    }

    /// Simulates the runway until assets are depleted.
    #[must_use]
    pub fn calculate_runway(&self) -> RunwayResult {
        let mut months = 0;
        let mut current_assets = self.liquid_assets_cents;
        #[allow(clippy::cast_precision_loss)]
        let mut current_burn = self.monthly_burn_cents as f64;
        let mut total_burned = 0;

        let monthly_inflation_rate = if self.annual_inflation_pct > 0.0 {
            // (1 + annual_rate)^(1/12) - 1
            (1.0 + self.annual_inflation_pct / 100.0).powf(1.0 / 12.0) - 1.0
        } else {
            0.0
        };

        while current_assets > 0 {
            #[allow(clippy::cast_possible_truncation)]
            let current_burn_cents = current_burn.round() as i64;

            if current_burn_cents == 0 {
                return RunwayResult {
                    months: 1200,
                    total_burned_cents: total_burned,
                };
            }

            let actual_burn = current_assets.min(current_burn_cents);
            current_assets -= actual_burn;
            total_burned += actual_burn;
            months += 1;

            if current_assets == 0 {
                break;
            }

            current_burn *= 1.0 + monthly_inflation_rate;
        }

        RunwayResult {
            months,
            total_burned_cents: total_burned,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_burn_rate() {
        let sim = RunwaySimulator::new(10_000_000, 0, 0.0);
        let result = sim.calculate_runway();
        // With 0 burn rate, runway is effectively infinite, capped at 1200 months
        assert_eq!(result.months, 1200);
    }

    #[test]
    fn test_simple_runway_no_inflation() {
        // $10k assets, $1k burn = 10 months
        let sim = RunwaySimulator::new(1_000_000, 100_000, 0.0);
        let result = sim.calculate_runway();
        assert_eq!(result.months, 10);
        assert_eq!(result.total_burned_cents, 1_000_000);
    }

    #[test]
    fn test_runway_with_inflation() {
        // $12k assets, $1k initial burn, 12% inflation.
        // Due to inflation, $1000/mo will increase each month.
        // It should still last exactly 12 months, but the total burned will equal the available assets
        // (the 12th month will be a partial burn, so we won't get a 13th month).
        let sim = RunwaySimulator::new(1_200_000, 100_000, 12.0);
        let result = sim.calculate_runway();
        assert_eq!(result.months, 12);
        assert_eq!(result.total_burned_cents, 1_200_000);

        // However, if we only have exactly enough for 11 months *without* inflation ($1.1M)
        let sim2 = RunwaySimulator::new(1_100_000, 100_000, 12.0);
        let result2 = sim2.calculate_runway();
        // Since burn rate increases, 11 months of burn will cost > $1.1M, so it will last < 11 months,
        // Actually, let's just assert that 11 months of assets lasts exactly 11 months
        // because the final month is a partial month.
        assert_eq!(result2.months, 11);

        // Let's test a case where inflation definitely drops a whole month.
        // If we have $11.5k, without inflation it lasts 11.5 months (so 12 months).
        // With high inflation, the 11th month might drain the rest.
        let sim3 = RunwaySimulator::new(1_150_000, 100_000, 50.0);
        let result3 = sim3.calculate_runway();
        assert!(result3.months <= 11);
    }
}
