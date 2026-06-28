//! Geo Arbitrage Simulator
//!
//! 🌟 Nova Feature: Calculate how relocating to different areas (geo-arbitrage)
//! accelerates or decelerates your path to FIRE. By evaluating location proposals,
//! you can see your adjusted FIRE number and how your progress changes instantly!

use crate::planning::fire::FireSimulator;

/// Represents a potential location to move to and its associated monthly expenses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocationProposal {
    pub name: String,
    pub estimated_monthly_expenses_cents: i64,
}

/// A report detailing how a location proposal affects your FIRE journey.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeoArbitrageReport {
    pub location_name: String,
    pub original_fire_number_cents: i64,
    pub adjusted_fire_number_cents: i64,
    pub original_progress_pct: u8,
    pub adjusted_progress_pct: u8,
}

/// Simulates the impact of geo-arbitrage on a base FIRE scenario.
#[derive(Debug, Clone)]
pub struct GeoArbitrageSimulator {
    base_simulator: FireSimulator,
}

impl GeoArbitrageSimulator {
    #[must_use]
    pub const fn new(base_simulator: FireSimulator) -> Self {
        Self { base_simulator }
    }

    /// Evaluates a location proposal and returns a report of the impact.
    #[must_use]
    pub fn evaluate_location(&self, proposal: &LocationProposal) -> GeoArbitrageReport {
        let original_fire_number = self.base_simulator.fire_number_cents();
        let original_progress = self.base_simulator.fire_progress_pct();

        // Since FireSimulator doesn't expose a mutator for expenses directly,
        // we create a new one with the proposed expenses, and copy over the state.
        let mut adj_sim = FireSimulator::new(proposal.estimated_monthly_expenses_cents);
        adj_sim.set_config(self.base_simulator.config());

        // We know safe_net_worth_cents uses liquid_assets_cents, liabilities, upcoming vests.
        // We can just query `safe_net_worth_cents()` and insert it as an asset!
        adj_sim.add_assets_liabilities(self.base_simulator.safe_net_worth_cents(), 0);

        let adjusted_fire_number = adj_sim.fire_number_cents();
        let adjusted_progress = adj_sim.fire_progress_pct();

        GeoArbitrageReport {
            location_name: proposal.name.clone(),
            original_fire_number_cents: original_fire_number,
            adjusted_fire_number_cents: adjusted_fire_number,
            original_progress_pct: original_progress,
            adjusted_progress_pct: adjusted_progress,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_geo_arbitrage_evaluation() {
        // Base scenario: HCOL area, $10,000/mo expenses, $1M net worth
        let mut base_sim = FireSimulator::new(1_000_000); // $10k/mo
        base_sim.add_assets_liabilities(100_000_000, 0); // $1M NW

        // Original FIRE number @ 4% SWR: $10,000 * 12 * 25 = $3,000,000
        // Progress: 1M / 3M = 33%

        let simulator = GeoArbitrageSimulator::new(base_sim.clone());

        // Proposal: Move to LCOL area, $4,000/mo expenses
        let proposal = LocationProposal {
            name: "Chiang Mai".to_string(),
            estimated_monthly_expenses_cents: 400_000, // $4k/mo
        };

        let report = simulator.evaluate_location(&proposal);

        assert_eq!(report.location_name, "Chiang Mai");
        assert_eq!(report.original_fire_number_cents, 300_000_000); // $3M
        assert_eq!(report.original_progress_pct, 33);

        // Adjusted FIRE number @ 4% SWR: $4,000 * 12 * 25 = $1,200,000
        assert_eq!(report.adjusted_fire_number_cents, 120_000_000); // $1.2M

        // Adjusted Progress: 1M / 1.2M = 83%
        assert_eq!(report.adjusted_progress_pct, 83);
    }
}
