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
    let total = projected_events_cents.iter().copied().sum::<i64>();

    RsuForecastSummary {
        event_count: projected_events_cents.len(),
        projected_total_cents: total,
    }
}
