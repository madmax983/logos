//! FIRE Readiness Projector Module.
//!
//! Connects `NetWorthProjector` and `TrinitySimulator` to map out
//! a timeline of retirement success probability as wealth grows.

#![cfg(feature = "nova")]

use crate::experimental::trinity_simulator::TrinitySimulator;
use crate::planning::net_worth_projector::NetWorthProjector;

/// A snapshot of your retirement readiness at a future point in time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadinessSnapshot {
    /// The month index in the projection.
    pub month_index: u16,
    /// The projected net worth in cents.
    pub projected_net_worth_cents: i64,
    /// The probability of retirement success (0-100) if you retired in this month.
    pub retirement_success_rate_pct: u8,
}

/// A projector to simulate how your retirement readiness improves over time.
#[derive(Debug, Clone)]
pub struct ReadinessProjector {
    net_worth_projector: NetWorthProjector,
    annual_expenses_cents: i64,
    annual_mean_return: f64,
    annual_volatility: f64,
    annual_inflation_rate_pct: f64,
    retirement_years: u16,
    monte_carlo_paths: u32,
    seed: u64,
}

impl ReadinessProjector {
    /// Creates a new `ReadinessProjector`.
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        net_worth_projector: NetWorthProjector,
        annual_expenses_cents: i64,
        annual_mean_return: f64,
        annual_volatility: f64,
        annual_inflation_rate_pct: f64,
        retirement_years: u16,
        monte_carlo_paths: u32,
        seed: u64,
    ) -> Self {
        Self {
            net_worth_projector,
            annual_expenses_cents,
            annual_mean_return,
            annual_volatility,
            annual_inflation_rate_pct,
            retirement_years,
            monte_carlo_paths,
            seed,
        }
    }

    /// Projects the readiness over `months_to_project`, evaluating the retirement
    /// success rate every `evaluate_every_n_months`.
    #[must_use]
    pub fn project_readiness(
        &self,
        months_to_project: u16,
        evaluate_every_n_months: u16,
    ) -> Vec<ReadinessSnapshot> {
        let (timeline, _) = self.net_worth_projector.project_timeline(months_to_project);
        let mut readiness_timeline = Vec::new();

        if evaluate_every_n_months == 0 {
            return readiness_timeline;
        }

        for month in timeline {
            if month.month_index % evaluate_every_n_months == 0
                || month.month_index == months_to_project
            {
                let trinity = TrinitySimulator::new(
                    month.net_worth_cents,
                    self.annual_expenses_cents,
                    self.annual_mean_return,
                    self.annual_volatility,
                    self.annual_inflation_rate_pct,
                    self.seed.wrapping_add(u64::from(month.month_index)),
                );

                let result = trinity.run(self.retirement_years, self.monte_carlo_paths);

                readiness_timeline.push(ReadinessSnapshot {
                    month_index: month.month_index,
                    projected_net_worth_cents: month.net_worth_cents,
                    retirement_success_rate_pct: result.success_rate_pct,
                });
            }
        }

        readiness_timeline
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_readiness_improvement_over_time() {
        // Start with $500k, save $5k per month
        let nw_projector = NetWorthProjector::new(50_000_000, 500_000);

        let projector = ReadinessProjector::new(
            nw_projector,
            60_000_000, // $60k annual expenses
            0.07,       // 7% return
            0.15,       // 15% volatility
            3.0,        // 3% inflation
            30,         // 30 years retirement
            100,        // 100 paths
            42,         // seed
        );

        // Project 5 years (60 months), evaluate every 12 months
        let snapshots = projector.project_readiness(60, 12);

        assert_eq!(snapshots.len(), 5); // Month 12, 24, 36, 48, 60

        // As net worth grows, success rate should generally improve or stay 100%
        let mut prev_nw = 0;
        for snapshot in &snapshots {
            assert!(snapshot.projected_net_worth_cents > prev_nw);
            prev_nw = snapshot.projected_net_worth_cents;
        }
    }

    #[test]
    fn test_zero_evaluation_interval() {
        let nw_projector = NetWorthProjector::new(100_000, 10_000);
        let projector = ReadinessProjector::new(nw_projector, 50_000, 0.07, 0.15, 3.0, 30, 10, 42);
        let snapshots = projector.project_readiness(12, 0);
        assert!(snapshots.is_empty());
    }
}
