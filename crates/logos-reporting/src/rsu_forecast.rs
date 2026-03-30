#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RsuForecastSummary {
    event_count: usize,
    projected_total_cents: i64,
}

impl RsuForecastSummary {
    #[must_use]
    pub const fn event_count(&self) -> usize {
        self.event_count
    }

    #[must_use]
    pub const fn projected_total_cents(&self) -> i64 {
        self.projected_total_cents
    }
}

#[must_use]
pub fn project_rsu_forecast_summary(projected_events_cents: &[i64]) -> RsuForecastSummary {
    let total = projected_events_cents.iter().copied().fold(0i64, i64::saturating_add);

    RsuForecastSummary {
        event_count: projected_events_cents.len(),
        projected_total_cents: total,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_rsu_forecast_summary() {
        let summary = project_rsu_forecast_summary(&[100, 200, 300]);
        assert_eq!(summary.event_count(), 3);
        assert_eq!(summary.projected_total_cents(), 600);
    }

    #[test]
    fn test_project_rsu_forecast_summary_empty() {
        let summary = project_rsu_forecast_summary(&[]);
        assert_eq!(summary.event_count(), 0);
        assert_eq!(summary.projected_total_cents(), 0);
    }

    #[test]
    fn test_project_rsu_forecast_summary_saturates_on_overflow() {
        let summary = project_rsu_forecast_summary(&[i64::MAX, 1]);
        assert_eq!(summary.event_count(), 2);
        assert_eq!(summary.projected_total_cents(), i64::MAX);

        let summary = project_rsu_forecast_summary(&[i64::MIN, -1]);
        assert_eq!(summary.event_count(), 2);
        assert_eq!(summary.projected_total_cents(), i64::MIN);
    }
}
