//! Geo-Arbitrage Simulator
//!
//! A simulator that calculates how moving to a different geographic region
//! affects your FIRE (Financial Independence, Retire Early) goals. It adjusts
//! your target FIRE number based on a Cost of Living (COL) multiplier for the region.

/// Represents a geographic region with an associated Cost of Living multiplier.
#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::derive_partial_eq_without_eq)]
pub struct GeoRegion {
    /// The name of the region (e.g., "Chiang Mai", "New York City").
    pub name: String,
    /// Cost of Living multiplier relative to your current location (1.0 = same, 0.5 = half cost).
    pub cost_of_living_multiplier: f64,
}

/// The result of simulating your FIRE goals in a specific region.
#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::derive_partial_eq_without_eq)]
pub struct RegionResult {
    /// The adjusted FIRE number in cents for this region.
    pub local_fire_number_cents: i64,
    /// Your progress percentage towards the FIRE number in this region (0 to 100).
    pub local_progress_pct: u8,
    /// The gap between your current net worth and the region's FIRE number in cents.
    /// If you have surpassed the goal, this is negative (a surplus).
    pub shortfall_cents: i64,
}

/// Simulates how your current financial situation maps to FIRE goals in different regions.
#[derive(Debug, Clone)]
pub struct GeoArbitrageSimulator {
    current_net_worth_cents: i64,
    monthly_expenses_cents: i64,
    safe_withdrawal_rate_pct: u8,
}

impl GeoArbitrageSimulator {
    /// Creates a new `GeoArbitrageSimulator`.
    ///
    /// # Arguments
    /// * `current_net_worth_cents` - Your current total net worth in cents.
    /// * `monthly_expenses_cents` - Your current monthly living expenses in cents.
    /// * `safe_withdrawal_rate_pct` - Your target safe withdrawal rate as an integer percentage (e.g., 4).
    #[must_use]
    pub const fn new(
        current_net_worth_cents: i64,
        monthly_expenses_cents: i64,
        safe_withdrawal_rate_pct: u8,
    ) -> Self {
        Self {
            current_net_worth_cents,
            monthly_expenses_cents,
            safe_withdrawal_rate_pct,
        }
    }

    /// Calculates the FIRE outlook if you were to relocate to the given region.
    #[must_use]
    pub fn simulate_region(&self, region: &GeoRegion) -> RegionResult {
        if self.safe_withdrawal_rate_pct == 0 {
            return RegionResult {
                local_fire_number_cents: i64::MAX,
                local_progress_pct: 0,
                shortfall_cents: i64::MAX,
            };
        }

        // Adjust the monthly expenses by the COL multiplier
        #[allow(clippy::cast_precision_loss)]
        let expenses_f64 = self.monthly_expenses_cents as f64;
        let adjusted_expenses = expenses_f64 * region.cost_of_living_multiplier;

        #[allow(clippy::cast_possible_truncation)]
        let adjusted_expenses_cents = adjusted_expenses.round() as i64;

        // Calculate the regional FIRE number
        // FIRE Number = (Monthly Expenses * 12 * 100) / SWR
        let local_fire_number_cents = adjusted_expenses_cents
            .saturating_mul(12)
            .saturating_mul(100)
            .saturating_div(i64::from(self.safe_withdrawal_rate_pct));

        // Calculate the progress percentage
        let progress_pct = if local_fire_number_cents == 0 {
            100
        } else if self.current_net_worth_cents <= 0 {
            0
        } else {
            let pct = (self.current_net_worth_cents.saturating_mul(100)) / local_fire_number_cents;
            std::cmp::min(100, pct).try_into().unwrap_or(100)
        };

        // Calculate the shortfall (or surplus if negative)
        let shortfall_cents = local_fire_number_cents.saturating_sub(self.current_net_worth_cents);

        RegionResult {
            local_fire_number_cents,
            local_progress_pct: progress_pct,
            shortfall_cents,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_geo_arbitrage_lower_col() {
        // Base: $500k NW, $5k/mo expenses, 4% SWR
        // Base FIRE number: $1.5M. Progress: 33%.
        let simulator = GeoArbitrageSimulator::new(50_000_000, 500_000, 4);

        // Region: Chiang Mai (50% cheaper)
        let region = GeoRegion {
            name: "Chiang Mai".to_string(),
            cost_of_living_multiplier: 0.5,
        };

        let result = simulator.simulate_region(&region);

        // Adjusted FIRE number: $750k
        assert_eq!(result.local_fire_number_cents, 75_000_000);

        // Progress: $500k / $750k = 66%
        assert_eq!(result.local_progress_pct, 66);

        // Shortfall: $750k - $500k = $250k
        assert_eq!(result.shortfall_cents, 25_000_000);
    }

    #[test]
    fn test_geo_arbitrage_higher_col() {
        // Base: $1M NW, $5k/mo expenses, 4% SWR
        // Base FIRE number: $1.5M. Progress: 66%.
        let simulator = GeoArbitrageSimulator::new(100_000_000, 500_000, 4);

        // Region: NYC (double the cost)
        let region = GeoRegion {
            name: "New York City".to_string(),
            cost_of_living_multiplier: 2.0,
        };

        let result = simulator.simulate_region(&region);

        // Adjusted FIRE number: $3M
        assert_eq!(result.local_fire_number_cents, 300_000_000);

        // Progress: $1M / $3M = 33%
        assert_eq!(result.local_progress_pct, 33);

        // Shortfall: $3M - $1M = $2M
        assert_eq!(result.shortfall_cents, 200_000_000);
    }

    #[test]
    fn test_geo_arbitrage_already_fire() {
        // Base: $2M NW, $5k/mo expenses, 4% SWR
        let simulator = GeoArbitrageSimulator::new(200_000_000, 500_000, 4);

        let region = GeoRegion {
            name: "Same City".to_string(),
            cost_of_living_multiplier: 1.0,
        };

        let result = simulator.simulate_region(&region);

        // Adjusted FIRE number: $1.5M
        assert_eq!(result.local_fire_number_cents, 150_000_000);

        // Progress: 100% (capped)
        assert_eq!(result.local_progress_pct, 100);

        // Shortfall: $1.5M - $2M = -$500k (Surplus)
        assert_eq!(result.shortfall_cents, -50_000_000);
    }

    #[test]
    fn test_geo_arbitrage_zero_swr() {
        let simulator = GeoArbitrageSimulator::new(100_000_000, 500_000, 0);

        let region = GeoRegion {
            name: "Anywhere".to_string(),
            cost_of_living_multiplier: 1.0,
        };

        let result = simulator.simulate_region(&region);

        assert_eq!(result.local_fire_number_cents, i64::MAX);
        assert_eq!(result.local_progress_pct, 0);
        assert_eq!(result.shortfall_cents, i64::MAX);
    }

    #[test]
    fn test_geo_arbitrage_zero_expenses() {
        let simulator = GeoArbitrageSimulator::new(100_000_000, 0, 4);

        let region = GeoRegion {
            name: "Free City".to_string(),
            cost_of_living_multiplier: 1.0,
        };

        let result = simulator.simulate_region(&region);

        assert_eq!(result.local_fire_number_cents, 0);
        assert_eq!(result.local_progress_pct, 100);
        // Shortfall: 0 - $1M = -$1M
        assert_eq!(result.shortfall_cents, -100_000_000);
    }

    #[test]
    fn test_geo_arbitrage_negative_net_worth() {
        let simulator = GeoArbitrageSimulator::new(-50_000_000, 500_000, 4);

        let region = GeoRegion {
            name: "Same City".to_string(),
            cost_of_living_multiplier: 1.0,
        };

        let result = simulator.simulate_region(&region);

        assert_eq!(result.local_fire_number_cents, 150_000_000);
        assert_eq!(result.local_progress_pct, 0);
        // Shortfall: $1.5M - (-$500k) = $2M
        assert_eq!(result.shortfall_cents, 200_000_000);
    }
}
