//! Geo-Arbitrage Simulator
//!
//! Simulates the financial impact of relocating to a different Cost of Living (`CoL`) area.

/// Represents a location with its specific tax rate and cost of living multiplier.
#[derive(Debug, Clone, PartialEq)]
pub struct LocationProfile {
    pub name: String,
    /// e.g., 1.0 for base, 0.8 for 20% cheaper
    pub col_multiplier: f64,
    /// e.g., 0.20 for 20% tax
    pub tax_rate: f64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArbitrageResult {
    pub original_savings: i64,
    pub new_savings: i64,
    pub delta: i64,
}

/// Simulates the impact of moving on annual savings.
#[derive(Debug, Default)]
pub struct GeoArbitrageSimulator;

impl GeoArbitrageSimulator {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Calculates the difference in annual savings if moving to the new location,
    /// assuming gross income remains identical but expenses and taxes change.
    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    pub fn simulate(
        &self,
        gross_income: i64,
        base_expenses: i64,
        current_location: &LocationProfile,
        new_location: &LocationProfile,
    ) -> ArbitrageResult {
        let current_tax = (gross_income as f64 * current_location.tax_rate) as i64;
        let current_expenses = (base_expenses as f64 * current_location.col_multiplier) as i64;
        let original_savings = gross_income - current_tax - current_expenses;

        let new_tax = (gross_income as f64 * new_location.tax_rate) as i64;
        let new_expenses = (base_expenses as f64 * new_location.col_multiplier) as i64;
        let new_savings = gross_income - new_tax - new_expenses;

        ArbitrageResult {
            original_savings,
            new_savings,
            delta: new_savings - original_savings,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_geo_arbitrage_positive_delta() {
        let sim = GeoArbitrageSimulator::new();
        let sf = LocationProfile {
            name: "San Francisco".into(),
            col_multiplier: 1.5,
            tax_rate: 0.35,
        };
        let austin = LocationProfile {
            name: "Austin".into(),
            col_multiplier: 0.9,
            tax_rate: 0.25,
        };

        // 200k gross, 50k base expenses
        let res = sim.simulate(20_000_000, 5_000_000, &sf, &austin);

        // SF: Tax 70k, Exp 75k -> Savings 55k
        assert_eq!(res.original_savings, 5_500_000);
        // Austin: Tax 50k, Exp 45k -> Savings 105k
        assert_eq!(res.new_savings, 10_500_000);
        assert_eq!(res.delta, 5_000_000);
    }
}
