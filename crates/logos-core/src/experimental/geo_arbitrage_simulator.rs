//! Geo-Arbitrage Simulator
//!
//! Simulates the financial impact of relocating to a new area.

/// Represents a potential location to move to.
#[derive(Debug, Clone, PartialEq)]
pub struct LocationProfile {
    pub name: String,
    pub col_multiplier: f64,
    pub income_multiplier: f64,
}

/// Represents the result of a geo-arbitrage simulation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeoArbitrageResult {
    pub location_name: String,
    pub new_annual_income_cents: i64,
    pub new_annual_expenses_cents: i64,
    pub new_annual_savings_cents: i64,
    pub new_fire_number_cents: i64,
    pub fire_number_difference_cents: i64,
}

/// A simulator for calculating the impact of geo-arbitrage.
#[derive(Debug, Clone)]
pub struct GeoArbitrageSimulator {
    current_annual_income_cents: i64,
    current_annual_expenses_cents: i64,
    safe_withdrawal_rate_pct: f64,
}

impl GeoArbitrageSimulator {
    #[must_use]
    pub const fn new(
        current_annual_income_cents: i64,
        current_annual_expenses_cents: i64,
        safe_withdrawal_rate_pct: f64,
    ) -> Self {
        Self {
            current_annual_income_cents,
            current_annual_expenses_cents,
            safe_withdrawal_rate_pct,
        }
    }

    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    pub fn simulate_relocation(&self, location: &LocationProfile) -> GeoArbitrageResult {
        let new_annual_income_cents =
            (self.current_annual_income_cents as f64 * location.income_multiplier) as i64;
        let new_annual_expenses_cents =
            (self.current_annual_expenses_cents as f64 * location.col_multiplier) as i64;
        let new_annual_savings_cents =
            new_annual_income_cents.saturating_sub(new_annual_expenses_cents);

        let current_fire_number_cents = (self.current_annual_expenses_cents as f64
            / (self.safe_withdrawal_rate_pct / 100.0))
            as i64;
        let new_fire_number_cents =
            (new_annual_expenses_cents as f64 / (self.safe_withdrawal_rate_pct / 100.0)) as i64;

        let fire_number_difference_cents =
            new_fire_number_cents.saturating_sub(current_fire_number_cents);

        GeoArbitrageResult {
            location_name: location.name.clone(),
            new_annual_income_cents,
            new_annual_expenses_cents,
            new_annual_savings_cents,
            new_fire_number_cents,
            fire_number_difference_cents,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_geo_arbitrage_to_cheaper_location_with_remote_job() {
        let sim = GeoArbitrageSimulator::new(150_000_00, 100_000_00, 4.0);
        let location = LocationProfile {
            name: "Bali".to_string(),
            col_multiplier: 0.4,
            income_multiplier: 1.0,
        };

        let result = sim.simulate_relocation(&location);

        assert_eq!(result.new_annual_income_cents, 150_000_00);
        assert_eq!(result.new_annual_expenses_cents, 40_000_00);
        assert_eq!(result.new_annual_savings_cents, 110_000_00);
        assert_eq!(result.new_fire_number_cents, 1_000_000_00);
        assert_eq!(result.fire_number_difference_cents, -1_500_000_00);
    }
}
