#![cfg(feature = "nova")]

//! FIRE Stress Tester
//!
//! Measures the time-delay impact of financial stress events (market crashes, job losses)
//! on the journey to Financial Independence.

use crate::planning::net_worth_projector::NetWorthProjector;

/// The result of a FIRE stress test comparison.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StressTestResult {
    /// Months to reach FIRE in the baseline scenario.
    pub baseline_months: Option<u16>,
    /// Months to reach FIRE in the stressed scenario.
    pub stressed_months: Option<u16>,
    /// The difference in months caused by the stress event.
    pub delay_months: Option<u16>,
}

/// Simulates financial shocks and compares them against a baseline projection.
#[derive(Debug, Clone)]
pub struct FireStressTester {
    baseline_projector: NetWorthProjector,
    fire_target_cents: i64,
    max_months: u16,
}

impl FireStressTester {
    /// Creates a new `FireStressTester`.
    #[must_use]
    pub const fn new(
        baseline_projector: NetWorthProjector,
        fire_target_cents: i64,
        max_months: u16,
    ) -> Self {
        Self {
            baseline_projector,
            fire_target_cents,
            max_months,
        }
    }

    /// Simulates a sudden percentage drop in net worth (e.g., a market crash).
    #[must_use]
    pub fn simulate_market_crash(&self, drop_pct: f64) -> StressTestResult {
        let baseline_months = self.get_months_to_target(&self.baseline_projector);

        let (initial_timeline, _) = self.baseline_projector.clone().project_timeline(1);
        let initial_nw = if initial_timeline.is_empty() {
            0
        } else {
            initial_timeline[0].net_worth_cents - initial_timeline[0].saved_cents
        };
        let monthly_savings = if initial_timeline.is_empty() {
            0
        } else {
            initial_timeline[0].saved_cents
        };

        let drop_factor = 1.0 - (drop_pct / 100.0);
        #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
        let stressed_initial_nw = (initial_nw as f64 * drop_factor).round() as i64;

        let mut stressed_projector = NetWorthProjector::new(stressed_initial_nw, monthly_savings);
        stressed_projector.add_milestone_cents(self.fire_target_cents);

        let stressed_months = self.get_months_to_target(&stressed_projector);

        StressTestResult {
            baseline_months,
            stressed_months,
            delay_months: match (baseline_months, stressed_months) {
                (Some(b), Some(s)) => Some(s.saturating_sub(b)),
                _ => None,
            },
        }
    }

    fn get_months_to_target(&self, projector: &NetWorthProjector) -> Option<u16> {
        let mut p = projector.clone();
        p.add_milestone_cents(self.fire_target_cents);
        let (_, crossed) = p.project_timeline(self.max_months);
        crossed
            .into_iter()
            .find(|(m, _)| *m == self.fire_target_cents)
            .map(|(_, month)| month)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_market_crash_delay() {
        let projector = NetWorthProjector::new(50_000_000, 1_000_000); // 500k initial, 10k monthly
        let tester = FireStressTester::new(projector, 100_000_000, 120); // 1M target

        let result = tester.simulate_market_crash(20.0); // 20% crash (drops to 400k)

        // Baseline: Needs 500k at 10k/mo = 50 months
        assert_eq!(result.baseline_months, Some(50));

        // Stressed: Needs 600k at 10k/mo = 60 months
        assert_eq!(result.stressed_months, Some(60));

        // Delay: 10 months
        assert_eq!(result.delay_months, Some(10));
    }
}
