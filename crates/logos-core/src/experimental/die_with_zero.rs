//! Die With Zero Simulator
//!
//! Calculates the optimal monthly spend rate to perfectly exhaust liquid assets
//! by a target life expectancy, factoring in real returns (investment return minus inflation).

/// Represents the simulation result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DieWithZeroResult {
    /// The optimal maximum monthly spend in cents to die with exactly zero assets.
    pub optimal_monthly_spend_cents: i64,
}

/// A simulator for calculating the "Die With Zero" optimal spend rate.
#[derive(Debug, Clone)]
pub struct DieWithZeroSimulator {
    current_age_years: u32,
    life_expectancy_years: u32,
    liquid_assets_cents: i64,
    annual_return_pct: f64,
    annual_inflation_pct: f64,
}

impl DieWithZeroSimulator {
    /// Creates a new `DieWithZeroSimulator`.
    #[must_use]
    pub const fn new(
        current_age_years: u32,
        life_expectancy_years: u32,
        liquid_assets_cents: i64,
        annual_return_pct: f64,
        annual_inflation_pct: f64,
    ) -> Self {
        Self {
            current_age_years,
            life_expectancy_years,
            liquid_assets_cents,
            annual_return_pct,
            annual_inflation_pct,
        }
    }

    /// Simulates and calculates the optimal spend using binary search.
    #[must_use]
    pub fn calculate_optimal_spend(&self) -> DieWithZeroResult {
        if self.current_age_years >= self.life_expectancy_years || self.liquid_assets_cents <= 0 {
            return DieWithZeroResult {
                optimal_monthly_spend_cents: 0,
            };
        }

        let months = (self.life_expectancy_years - self.current_age_years) * 12;
        let return_factor = 1.0 + self.annual_return_pct / 100.0;
        let inflation_factor = 1.0 + self.annual_inflation_pct / 100.0;

        let monthly_real_return = (return_factor / inflation_factor).powf(1.0 / 12.0) - 1.0;

        let mut low: i64 = 0;
        let mut high: i64 = self.liquid_assets_cents.max(10_000_000_000); // Start high enough
        let mut best: i64 = 0;

        for _ in 0..100 { // Max iterations for precision
            let mid = low + (high - low) / 2;
            let mut current_assets = self.liquid_assets_cents as f64;
            let mut failed = false;

            for _ in 0..months {
                current_assets -= mid as f64;
                if current_assets < 0.0 {
                    failed = true;
                    break;
                }
                current_assets *= 1.0 + monthly_real_return;
            }

            if failed {
                high = mid;
            } else {
                best = mid;
                low = mid + 1;
            }
        }

        DieWithZeroResult {
            optimal_monthly_spend_cents: best,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_assets() {
        let sim = DieWithZeroSimulator::new(30, 80, 0, 7.0, 3.0);
        let result = sim.calculate_optimal_spend();
        assert_eq!(result.optimal_monthly_spend_cents, 0);
    }

    #[test]
    fn test_age_exceeds_expectancy() {
        let sim = DieWithZeroSimulator::new(85, 80, 100_000_000, 7.0, 3.0);
        let result = sim.calculate_optimal_spend();
        assert_eq!(result.optimal_monthly_spend_cents, 0);
    }

    #[test]
    fn test_simple_spend_no_returns() {
        // 1M over 10 years (120 months) = 8333.33/mo
        let sim = DieWithZeroSimulator::new(70, 80, 100_000_000, 0.0, 0.0);
        let result = sim.calculate_optimal_spend();
        // Allow a small rounding tolerance
        assert!((result.optimal_monthly_spend_cents - 833_333).abs() <= 1);
    }

    #[test]
    fn test_spend_with_returns() {
        // Real return is positive, should allow higher spend than no returns
        let sim1 = DieWithZeroSimulator::new(30, 80, 100_000_000, 0.0, 0.0);
        let sim2 = DieWithZeroSimulator::new(30, 80, 100_000_000, 7.0, 3.0);
        let result1 = sim1.calculate_optimal_spend();
        let result2 = sim2.calculate_optimal_spend();
        assert!(result2.optimal_monthly_spend_cents > result1.optimal_monthly_spend_cents);
    }
}
