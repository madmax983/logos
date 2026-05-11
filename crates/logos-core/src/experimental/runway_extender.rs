#![cfg(feature = "nova")]

//! Runway Extender Module
//!
//! # The Mashup
//!
//! Connects the `RunwaySimulator` with the `SubscriptionFatigueReport`.
//! This answers a simple but powerful question: "If I cancel all these annoying
//! recurring subscriptions, exactly how many extra months of freedom (runway)
//! do I buy myself?"

use crate::experimental::runway_simulator::RunwaySimulator;
use crate::experimental::subscription_fatigue::SubscriptionFatigueReport;

/// The result of calculating the runway extension.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunwayExtensionResult {
    /// The original runway in months.
    pub original_months: u32,
    /// The new runway in months after cutting subscriptions.
    pub new_months: u32,
    /// The number of additional months gained.
    pub months_gained: u32,
}

/// Extends a financial runway by analyzing subscription fatigue.
#[derive(Debug, Clone)]
pub struct RunwayExtender {
    base_simulator: RunwaySimulator,
    fatigue_report: SubscriptionFatigueReport,
}

impl RunwayExtender {
    /// Creates a new `RunwayExtender`.
    #[must_use]
    pub const fn new(
        base_simulator: RunwaySimulator,
        fatigue_report: SubscriptionFatigueReport,
    ) -> Self {
        Self {
            base_simulator,
            fatigue_report,
        }
    }

    /// Calculates the impact of cancelling all subscriptions in the fatigue report.
    #[must_use]
    pub fn calculate_extension(&self) -> RunwayExtensionResult {
        let original_result = self.base_simulator.calculate_runway();

        // We need a new simulator with the reduced burn rate
        let new_burn = self
            .base_simulator
            .monthly_burn_cents()
            .saturating_sub(self.fatigue_report.total_monthly_cost_cents);
        let safe_new_burn = if new_burn < 0 { 0 } else { new_burn };

        let new_simulator = RunwaySimulator::new(
            self.base_simulator.liquid_assets_cents(),
            safe_new_burn,
            self.base_simulator.annual_inflation_pct(),
        );

        let new_result = new_simulator.calculate_runway();

        RunwayExtensionResult {
            original_months: original_result.months,
            new_months: new_result.months,
            months_gained: new_result.months.saturating_sub(original_result.months),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runway_extension() {
        // $12k assets, $2k burn. Original runway = 6 months.
        let sim = RunwaySimulator::new(1_200_000, 200_000, 0.0);

        // Cancel $1k of subscriptions. New burn = $1k. New runway = 12 months.
        let report = SubscriptionFatigueReport {
            items: vec![],
            total_monthly_cost_cents: 100_000,
            total_opportunity_cost_cents: 0,
        };

        let extender = RunwayExtender::new(sim, report);
        let result = extender.calculate_extension();

        assert_eq!(result.original_months, 6);
        assert_eq!(result.new_months, 12);
        assert_eq!(result.months_gained, 6);
    }

    #[test]
    fn test_runway_extension_zero_burn() {
        let sim = RunwaySimulator::new(1_200_000, 100_000, 0.0);

        // Cancel $1k of subscriptions. New burn = $0. Runway capped at 1200 months.
        let report = SubscriptionFatigueReport {
            items: vec![],
            total_monthly_cost_cents: 100_000,
            total_opportunity_cost_cents: 0,
        };

        let extender = RunwayExtender::new(sim, report);
        let result = extender.calculate_extension();

        assert_eq!(result.original_months, 12);
        assert_eq!(result.new_months, 1200);
        assert_eq!(result.months_gained, 1188);
    }
}
