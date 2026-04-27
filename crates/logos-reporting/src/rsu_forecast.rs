//! # RSU Forecasting
//!
//! Provides aggregation metrics for projected RSU vesting events.
//! This module summarizes a series of discrete future liquidity events
//! into high-level totals for reporting.

/// A high-level summary of a series of projected RSU vesting events.
///
/// This struct aggregates the raw projected value of multiple upcoming
/// vesting events into a single, comprehensive overview.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RsuForecastSummary {
    event_count: usize,
    projected_total_cents: i64,
}

impl RsuForecastSummary {
    /// The number of distinct vesting dates covered by this projection window.
    /// This is useful for UI reporting to indicate how many discrete liquidity events
    /// are contributing to the total forecast.
    ///
    /// ## Examples
    ///
    /// ```
    /// use logos_reporting::project_rsu_forecast_summary;
    ///
    /// let summary = project_rsu_forecast_summary(&[100_00, 200_00]);
    /// assert_eq!(summary.event_count(), 2);
    /// ```
    #[must_use]
    pub const fn event_count(&self) -> usize {
        self.event_count
    }

    /// The aggregated gross value in cents of all anticipated vesting events.
    /// This represents the total theoretical pre-tax value if all units vest
    /// and are sold at the projected price scenarios.
    ///
    /// ## Examples
    ///
    /// ```
    /// use logos_reporting::project_rsu_forecast_summary;
    ///
    /// let summary = project_rsu_forecast_summary(&[100_00, 200_00]);
    /// assert_eq!(summary.projected_total_cents(), 300_00);
    /// ```
    #[must_use]
    pub const fn projected_total_cents(&self) -> i64 {
        self.projected_total_cents
    }
}

/// Aggregates a slice of projected individual event values into a [`RsuForecastSummary`].
///
/// ## Examples
///
/// ```
/// use logos_reporting::project_rsu_forecast_summary;
///
/// let projected_events = vec![500_00, 250_00, 250_00];
/// let summary = project_rsu_forecast_summary(&projected_events);
///
/// assert_eq!(summary.event_count(), 3);
/// assert_eq!(summary.projected_total_cents(), 1000_00);
/// ```
#[must_use]
pub fn project_rsu_forecast_summary(projected_events_cents: &[i64]) -> RsuForecastSummary {
    let total = projected_events_cents
        .iter()
        .copied()
        .fold(0_i64, i64::saturating_add);

    RsuForecastSummary {
        event_count: projected_events_cents.len(),
        projected_total_cents: total,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_rsu_forecast_summary_saturates_on_overflow() {
        let events = [i64::MAX, 1];
        let summary = project_rsu_forecast_summary(&events);
        assert_eq!(summary.projected_total_cents(), i64::MAX);

        let events_under = [i64::MIN, -1];
        let summary_under = project_rsu_forecast_summary(&events_under);
        assert_eq!(summary_under.projected_total_cents(), i64::MIN);
    }
}
