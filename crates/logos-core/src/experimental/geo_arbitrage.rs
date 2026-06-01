#![cfg(feature = "nova")]
//! Geo-Arbitrage Simulator
//!
//! A simulator to model the financial impact of relocating to a different
//! cost-of-living area. It compares the FIRE timeline of a current scenario
//! against a relocated scenario where monthly savings are adjusted by a
//! cost-of-living multiplier.

use crate::planning::fire::UpcomingVest;
use crate::planning::net_worth_projector::NetWorthProjector;

/// Configuration for the current location scenario.
#[derive(Debug, Clone)]
pub struct CurrentScenario {
    pub initial_net_worth_cents: i64,
    pub monthly_income_cents: i64,
    pub monthly_savings_cents: i64,
    pub upcoming_vests: Vec<UpcomingVest>,
    pub fire_milestone_cents: i64,
}

/// A target location with a cost-of-living multiplier.
#[derive(Debug, Clone, PartialEq)]
pub struct Location {
    pub name: String,
    /// Multiplier applied to expenses. A value < 1.0 means cheaper.
    pub col_multiplier: f64,
    /// Total monthly income (assumed constant for simplicity).
    pub monthly_income_cents: i64,
}

/// Result of comparing a target location against the current scenario.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeoArbitrageResult {
    pub location_name: String,
    pub current_months_to_fire: Option<u16>,
    pub relocated_months_to_fire: Option<u16>,
}

impl GeoArbitrageResult {
    /// The number of months saved (or lost) by relocating.
    /// A positive number indicates FIRE is reached sooner.
    #[must_use]
    pub fn months_saved(&self) -> Option<i32> {
        match (self.current_months_to_fire, self.relocated_months_to_fire) {
            (Some(curr), Some(reloc)) => Some(i32::from(curr) - i32::from(reloc)),
            _ => None,
        }
    }
}

/// Simulator to evaluate multiple target locations.
#[derive(Debug, Clone)]
pub struct GeoArbitrageSimulator {
    current_scenario: CurrentScenario,
    max_projection_months: u16,
}

impl GeoArbitrageSimulator {
    /// Creates a new `GeoArbitrageSimulator`.
    #[must_use]
    pub const fn new(current_scenario: CurrentScenario, max_projection_months: u16) -> Self {
        Self {
            current_scenario,
            max_projection_months,
        }
    }

    /// Simulates the FIRE timeline for a target location and compares it to the current scenario.
    #[must_use]
    pub fn evaluate(&self, target_location: &Location) -> GeoArbitrageResult {
        let current_months_to_fire =
            self.simulate_scenario(self.current_scenario.monthly_savings_cents);

        // Calculate relocated monthly savings.
        // Savings = Income - Expenses.
        // Expenses = Current Income - Current Savings.
        // Relocated Expenses = Expenses * col_multiplier.
        // Relocated Savings = Target Income - Relocated Expenses.
        let current_expenses =
            self.current_scenario.monthly_income_cents - self.current_scenario.monthly_savings_cents;

        #[allow(clippy::cast_precision_loss)]
        let current_expenses_f64 = current_expenses as f64;
        let relocated_expenses_f64 = current_expenses_f64 * target_location.col_multiplier;

        #[allow(clippy::cast_possible_truncation)]
        let relocated_expenses = relocated_expenses_f64.round() as i64;

        let relocated_savings = target_location.monthly_income_cents - relocated_expenses;

        let relocated_months_to_fire = self.simulate_scenario(relocated_savings);

        GeoArbitrageResult {
            location_name: target_location.name.clone(),
            current_months_to_fire,
            relocated_months_to_fire,
        }
    }

    fn simulate_scenario(&self, monthly_savings_cents: i64) -> Option<u16> {
        let mut projector = NetWorthProjector::new(
            self.current_scenario.initial_net_worth_cents,
            monthly_savings_cents,
        );

        for vest in &self.current_scenario.upcoming_vests {
            projector.add_upcoming_vest(*vest);
        }

        projector.add_milestone_cents(self.current_scenario.fire_milestone_cents);

        let (_, milestones) = projector.project_timeline(self.max_projection_months);

        milestones.first().map(|&(_, month)| month)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_geo_arbitrage_savings() {
        let scenario = CurrentScenario {
            initial_net_worth_cents: 10_000_000, // $100k
            monthly_income_cents: 500_000,      // $5k
            monthly_savings_cents: 200_000,     // $2k
            upcoming_vests: vec![],
            fire_milestone_cents: 20_000_000, // $200k
        };

        let simulator = GeoArbitrageSimulator::new(scenario, 100);

        // Current income: $5k/mo, Savings: $2k/mo, Expenses: $3k/mo.
        let cheap_city = Location {
            name: "Cheap City".to_string(),
            col_multiplier: 0.5, // Expenses become $1.5k/mo.
            monthly_income_cents: 500_000,
        };

        let result = simulator.evaluate(&cheap_city);

        // Relocated Savings = $5k - $1.5k = $3.5k/mo.
        // Current FIRE: ($200k - $100k) / $2k = 50 months.
        // Relocated FIRE: ($200k - $100k) / $3.5k = ~28.5 -> 29 months.

        assert_eq!(result.location_name, "Cheap City");
        assert_eq!(result.current_months_to_fire, Some(50));
        assert_eq!(result.relocated_months_to_fire, Some(29));
        assert_eq!(result.months_saved(), Some(21));
    }

    #[test]
    fn test_geo_arbitrage_expensive_city() {
        let scenario = CurrentScenario {
            initial_net_worth_cents: 5_000_000,
            monthly_income_cents: 800_000,
            monthly_savings_cents: 300_000,
            upcoming_vests: vec![],
            fire_milestone_cents: 10_000_000,
        };

        let simulator = GeoArbitrageSimulator::new(scenario, 60);

        let expensive_city = Location {
            name: "Expensive City".to_string(),
            col_multiplier: 1.5,
            monthly_income_cents: 800_000,
        };

        let result = simulator.evaluate(&expensive_city);

        assert_eq!(result.current_months_to_fire, Some(17));
        // Expenses = 8k - 3k = 5k.
        // Relocated Expenses = 5k * 1.5 = 7.5k.
        // Relocated Savings = 8k - 7.5k = 0.5k.
        // (100k - 50k) / 0.5k = 100 months. Max projection is 60.
        assert_eq!(result.relocated_months_to_fire, None);
        assert_eq!(result.months_saved(), None);
    }
}
