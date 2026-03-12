//! Net worth projection and milestone tracking over time.
//!
//! # The Time Machine
//!
//! This module contains primitives to simulate how a user's net worth will grow
//! over a period of months, factoring in monthly cash savings and upcoming RSU vests.
//! It answers the question: *When will I cross the finish line?*
//!
//! While the `FireSimulator` tells you what your target is, the `NetWorthProjector`
//! tells you the exact month you will hit it. It acts as a financial time machine,
//! moving forward month by month, collecting your steady savings, and waiting for
//! the volatile, risk-adjusted payouts of your company stock.
//!
//! It also identifies exactly when specific financial milestones (like a FIRE number)
//! will be achieved.

use crate::domain::rsu::{HaircutTierTable, forecast_value_cents};
use crate::planning::fire::UpcomingVest;

/// Represents a single month's snapshot in a net worth projection timeline.
///
/// This struct holds the aggregated financial state at the end of a given month
/// in the simulation.
///
/// ## Examples
///
/// ```
/// use logos_core::planning::net_worth_projector::ProjectedMonth;
///
/// let snapshot = ProjectedMonth {
///     month_index: 3,
///     net_worth_cents: 150_000_00, // $150k
///     vested_value_cents: 10_000_00, // $10k vested this month
///     saved_cents: 5_000_00, // $5k saved this month
/// };
///
/// assert_eq!(snapshot.month_index, 3);
/// assert_eq!(snapshot.net_worth_cents, 150_000_00);
/// ```
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
/// will be reached. The simulation runs forward month-by-month, treating time as a
/// linear progression of 30-day blocks. It patiently accumulates your boring, reliable
/// savings and waits for the exciting, risk-adjusted pops of your unvested RSUs.
///
/// Use this when you need a timeline, not just a target.
///
/// ## Examples
///
/// ```
/// use logos_core::planning::net_worth_projector::NetWorthProjector;
/// use logos_core::planning::fire::UpcomingVest;
///
/// // Start with $100k net worth, saving $5k per month.
/// let mut projector = NetWorthProjector::new(100_000_00, 5_000_00);
///
/// // Add a goal to track when we hit $150k.
/// projector.add_milestone_cents(150_000_00);
///
/// // Add a $50k vest happening in 45 days (Month 2).
/// projector.add_upcoming_vest(UpcomingVest {
///     avg_close_price_cents: 10_000,
///     units: 500, // 500 * $100 = $50k
///     days_to_vest: 45,
/// });
///
/// // Project 3 months into the future.
/// let (timeline, milestones) = projector.project_timeline(3);
/// assert_eq!(timeline.len(), 3);
/// ```
#[derive(Debug, Clone)]
pub struct NetWorthProjector {
    initial_net_worth_cents: i64,
    monthly_savings_cents: i64,
    upcoming_vests: Vec<UpcomingVest>,
    haircut_tiers: HaircutTierTable,
    milestones_cents: Vec<i64>,
}

impl NetWorthProjector {
    /// Creates a new `NetWorthProjector` to start simulating financial progress.
    ///
    /// ## Examples
    ///
    /// ```
    /// use logos_core::planning::net_worth_projector::NetWorthProjector;
    ///
    /// // Simulate starting with $50k, adding $2k every month.
    /// let projector = NetWorthProjector::new(50_000_00, 2_000_00);
    /// ```
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

    /// Updates the `HaircutTierTable` used to discount future RSU vests.
    ///
    /// The projector uses these tiers to determine how much "safe" value
    /// a future vest will add to the net worth. This is your reality check mechanism:
    /// it prevents you from counting your chickens (unvested RSUs) before they hatch,
    /// applying heavier discounts to vests that are further out in the uncertain future.
    ///
    /// ## Examples
    ///
    /// ```
    /// use logos_core::planning::net_worth_projector::NetWorthProjector;
    /// use logos_core::HaircutTierTable;
    ///
    /// let mut projector = NetWorthProjector::new(50_000_00, 2_000_00);
    /// let custom_tiers = HaircutTierTable::new(10, 20, 30).unwrap();
    /// projector.set_haircut_tiers(custom_tiers);
    /// ```
    pub const fn set_haircut_tiers(&mut self, tiers: HaircutTierTable) {
        self.haircut_tiers = tiers;
    }

    /// Registers an `UpcomingVest` to be included in the projection timeline.
    ///
    /// Vests are applied in the month they are scheduled to occur, where each
    /// month is modeled as a 30-day window.
    ///
    /// ## Examples
    ///
    /// ```
    /// use logos_core::planning::net_worth_projector::NetWorthProjector;
    /// use logos_core::planning::fire::UpcomingVest;
    ///
    /// let mut projector = NetWorthProjector::new(50_000_00, 2_000_00);
    /// projector.add_upcoming_vest(UpcomingVest {
    ///     avg_close_price_cents: 10_000, // $100 per share
    ///     units: 100, // 100 shares
    ///     days_to_vest: 60, // vests in 2 months
    /// });
    /// ```
    pub fn add_upcoming_vest(&mut self, vest: UpcomingVest) {
        self.upcoming_vests.push(vest);
    }

    /// Adds a financial milestone target in cents to track in the projection.
    ///
    /// The projector will output the exact month when the net worth crosses this value.
    ///
    /// ## Examples
    ///
    /// ```
    /// use logos_core::planning::net_worth_projector::NetWorthProjector;
    ///
    /// let mut projector = NetWorthProjector::new(50_000_00, 2_000_00);
    /// projector.add_milestone_cents(100_000_00); // track when we hit $100k
    /// ```
    pub fn add_milestone_cents(&mut self, milestone_cents: i64) {
        self.milestones_cents.push(milestone_cents);
    }

    /// Simulates net worth month-by-month for `months` iterations.
    ///
    /// This is the core engine of the projector, the time machine's ignition switch.
    /// It aggregates your steady savings and the risk-adjusted, safe value of your
    /// vesting stock into your total net worth. As it travels forward, it rings a bell
    /// (records the month) every time you smash through one of your financial milestones.
    ///
    /// Returns a tuple containing:
    /// 1. A timeline of [`ProjectedMonth`] snapshots detailing the journey.
    /// 2. A vector of tuples `(milestone_cents, month_index)` indicating the exact month each milestone was crossed.
    ///
    /// ## Examples
    ///
    /// ```
    /// use logos_core::planning::net_worth_projector::NetWorthProjector;
    ///
    /// let mut projector = NetWorthProjector::new(10_000, 1_000);
    /// projector.add_milestone_cents(12_000); // Want to reach $120.00
    ///
    /// // After 3 months of saving $10.00/month, we will have $130.00.
    /// let (timeline, milestones) = projector.project_timeline(3);
    ///
    /// assert_eq!(timeline.last().unwrap().net_worth_cents, 13_000);
    /// assert_eq!(milestones[0], (12_000, 2)); // Crossed $120.00 in month 2
    /// ```
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
            let month_start_days = (month_index - 1) * 30;
            let month_end_days = month_index * 30;

            for vest in &self.upcoming_vests {
                // If the vest falls in the current month's window
                if vest.days_to_vest > month_start_days && vest.days_to_vest <= month_end_days {
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
