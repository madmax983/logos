use crate::domain::rsu::{HaircutTierTable, forecast_value_cents};
use crate::planning::fire::UpcomingVest;

/// Represents a single month's snapshot in a net worth projection timeline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectedMonth {
    /// The month index in the projection (0 is the current month).
    pub month_index: u16,
    /// The net worth in cents at the end of this month.
    pub net_worth_cents: i64,
    /// Total safe value of RSUs that vested during this month, in cents.
    pub vested_value_cents: i64,
    /// Total cash saved during this month, in cents.
    pub saved_cents: i64,
}

/// Projects net worth over time based on steady savings and upcoming RSU vests.
///
/// This provides a crystal ball to see *when* financial milestones (e.g., FIRE number)
/// will be reached.
#[derive(Debug, Clone)]
pub struct NetWorthProjector {
    initial_net_worth_cents: i64,
    monthly_savings_cents: i64,
    upcoming_vests: Vec<UpcomingVest>,
    haircut_tiers: HaircutTierTable,
    milestones_cents: Vec<i64>,
}

impl NetWorthProjector {
    #[must_use]
    pub fn new(initial_net_worth_cents: i64, monthly_savings_cents: i64) -> Self {
        Self {
            initial_net_worth_cents,
            monthly_savings_cents,
            upcoming_vests: Vec::new(),
            haircut_tiers: HaircutTierTable::default(),
            milestones_cents: Vec::new(),
        }
    }

    pub const fn set_haircut_tiers(&mut self, tiers: HaircutTierTable) {
        self.haircut_tiers = tiers;
    }

    pub fn add_upcoming_vest(&mut self, vest: UpcomingVest) {
        self.upcoming_vests.push(vest);
    }

    pub fn add_milestone_cents(&mut self, milestone_cents: i64) {
        self.milestones_cents.push(milestone_cents);
    }

    /// Simulates net worth month-by-month for `months` iterations.
    ///
    /// Returns a tuple containing:
    /// 1. A timeline of `ProjectedMonth` snapshots.
    /// 2. A vector of tuples `(milestone_cents, month_index)` indicating the month each milestone was crossed.
    #[must_use]
    pub fn project_timeline(&self, months: u16) -> (Vec<ProjectedMonth>, Vec<(i64, u16)>) {
        let mut timeline = Vec::with_capacity(usize::from(months));
        let mut crossed_milestones = Vec::new();

        let mut current_net_worth = self.initial_net_worth_cents;

        // Keep track of which milestones have been crossed
        let mut uncrossed_milestones = self.milestones_cents.clone();
        uncrossed_milestones.sort_unstable(); // Sort so we cross smaller milestones first

        for month_index in 1..=months {
            let mut vested_this_month: i64 = 0;

            // Assume 1 month is roughly 30 days. We check if any vest happens in this 30-day window.
            let month_start_days = u32::from(month_index - 1) * 30;
            let month_end_days = u32::from(month_index) * 30;

            for vest in &self.upcoming_vests {
                // If the vest falls in the current month's window
                let vest_days = u32::from(vest.days_to_vest);
                if vest_days > month_start_days && vest_days <= month_end_days {
                    let safe_value = forecast_value_cents(
                        vest.avg_close_price_cents,
                        vest.units,
                        vest.days_to_vest,
                        &self.haircut_tiers,
                    );
                    vested_this_month = vested_this_month.saturating_add(safe_value);
                }
            }

            current_net_worth = current_net_worth
                .saturating_add(self.monthly_savings_cents)
                .saturating_add(vested_this_month);

            // Check for crossed milestones
            uncrossed_milestones.retain(|&milestone| {
                if current_net_worth >= milestone {
                    crossed_milestones.push((milestone, month_index));
                    false // Remove from uncrossed
                } else {
                    true // Keep in uncrossed
                }
            });

            timeline.push(ProjectedMonth {
                month_index,
                net_worth_cents: current_net_worth,
                vested_value_cents: vested_this_month,
                saved_cents: self.monthly_savings_cents,
            });
        }

        (timeline, crossed_milestones)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_timeline_no_vests() {
        let mut projector = NetWorthProjector::new(100_000, 10_000);
        projector.add_milestone_cents(115_000);
        projector.add_milestone_cents(130_000);

        let (timeline, crossed_milestones) = projector.project_timeline(5);

        assert_eq!(timeline.len(), 5);

        // Month 1
        assert_eq!(timeline[0].month_index, 1);
        assert_eq!(timeline[0].net_worth_cents, 110_000);
        assert_eq!(timeline[0].vested_value_cents, 0);

        // Month 2
        assert_eq!(timeline[1].month_index, 2);
        assert_eq!(timeline[1].net_worth_cents, 120_000);

        // Crossed milestones
        assert_eq!(crossed_milestones.len(), 2);
        assert_eq!(crossed_milestones[0], (115_000, 2)); // Crossed 115k in month 2 (120k > 115k)
        assert_eq!(crossed_milestones[1], (130_000, 3)); // Crossed 130k in month 3 (130k >= 130k)
    }

    #[test]
    fn test_project_timeline_with_vests() {
        let mut projector = NetWorthProjector::new(50_000, 5_000); // 500 initial, +50 per month

        // Add a vest that happens in month 2 (e.g. 45 days)
        projector.add_upcoming_vest(UpcomingVest {
            avg_close_price_cents: 10_000,
            units: 10,        // 100,000 cents gross
            days_to_vest: 45, // Month 2 (30 < days <= 60)
        });

        projector.add_milestone_cents(100_000);

        // Default haircut for 45 days is medium tier (40%), so safe value retained is 60%.
        // Safe value = 60,000 cents.

        let (timeline, crossed_milestones) = projector.project_timeline(3);

        assert_eq!(timeline.len(), 3);

        // Month 1: 50,000 + 5,000 = 55,000
        assert_eq!(timeline[0].net_worth_cents, 55_000);
        assert_eq!(timeline[0].vested_value_cents, 0);

        // Month 2: 55,000 + 5,000 (savings) + 60,000 (vest) = 120,000
        assert_eq!(timeline[1].net_worth_cents, 120_000);
        assert_eq!(timeline[1].vested_value_cents, 60_000);

        // Month 3: 120,000 + 5,000 = 125,000
        assert_eq!(timeline[2].net_worth_cents, 125_000);
        assert_eq!(timeline[2].vested_value_cents, 0);

        // Milestones
        assert_eq!(crossed_milestones.len(), 1);
        assert_eq!(crossed_milestones[0], (100_000, 2)); // Crossed 100k in month 2
    }
}
