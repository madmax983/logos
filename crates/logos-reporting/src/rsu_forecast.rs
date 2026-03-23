//! Restricted Stock Unit (RSU) forecast aggregation.
//!
//! # The Horizon
//!
//! When planning for the future, knowing the risk-adjusted value of a single RSU vest
//! is helpful, but seeing the aggregate total across all unvested grants provides the
//! true picture of upcoming cash flow.
//!
//! This module provides the structures to summarize a series of projected RSU events
//! into a single, high-level forecast, allowing you to answer "How much safe value
//! do I have vesting in total?"

/// A high-level summary of multiple projected RSU vest events.
///
/// It aggregates the individual safe values into a total projected value
/// while keeping track of the number of vesting events.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RsuForecastSummary {
    event_count: usize,
    projected_total_cents: i64,
}

impl RsuForecastSummary {
    /// The total number of vest events included in this forecast.
    #[must_use]
    pub const fn event_count(&self) -> usize {
        self.event_count
    }

    /// The combined, risk-adjusted value of all vest events in cents.
    #[must_use]
    pub const fn projected_total_cents(&self) -> i64 {
        self.projected_total_cents
    }
}

/// Projects a unified forecast summary from a collection of individual event values.
///
/// This function sums the provided projected values. It expects the values to have
/// already been risk-adjusted (e.g., via haircut tiers) prior to aggregation.
///
/// ## Examples
///
/// ```
/// use logos_reporting::project_rsu_forecast_summary;
///
/// // Three upcoming vests with safe projected values of $1,000, $1,500, and $500.
/// let events = vec![100_000, 150_000, 50_000];
///
/// let summary = project_rsu_forecast_summary(&events);
///
/// // We expect 3 total events summing to $3,000.
/// assert_eq!(summary.event_count(), 3);
/// assert_eq!(summary.projected_total_cents(), 300_000);
/// ```
#[must_use]
pub fn project_rsu_forecast_summary(projected_events_cents: &[i64]) -> RsuForecastSummary {
    // Note: If the number of vests and their values are extraordinarily large,
    // a saturating sum might be safer, but `sum()` is fine for standard financial models.
    let total = projected_events_cents
        .iter()
        .copied()
        .fold(0_i64, |acc, x| acc.saturating_add(x));

    RsuForecastSummary {
        event_count: projected_events_cents.len(),
        projected_total_cents: total,
    }
}
