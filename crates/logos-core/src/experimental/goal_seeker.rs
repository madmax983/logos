//! Financial Goal Seeker
//!
//! A simulator that uses binary search to reverse-engineer the minimum required
//! monthly savings rate needed to reach a target net worth within a specific
//! timeframe. It leverages the `NetWorthProjector` to account for upcoming
//! RSU vests and risk haircuts.

use crate::domain::rsu::HaircutTierTable;
use crate::planning::fire::UpcomingVest;
use crate::planning::net_worth_projector::NetWorthProjector;

/// An optimizer that finds the required monthly savings to hit a financial goal.
#[derive(Debug, Clone)]
pub struct GoalSeeker {
    initial_net_worth_cents: i64,
    upcoming_vests: Vec<UpcomingVest>,
    haircut_tiers: Option<HaircutTierTable>,
}

impl GoalSeeker {
    /// Creates a new `GoalSeeker` with the current financial baseline.
    #[must_use]
    pub const fn new(initial_net_worth_cents: i64) -> Self {
        Self {
            initial_net_worth_cents,
            upcoming_vests: Vec::new(),
            haircut_tiers: None,
        }
    }

    /// Adds an upcoming RSU vest to be included in the projections.
    pub fn add_upcoming_vest(&mut self, vest: UpcomingVest) {
        self.upcoming_vests.push(vest);
    }

    /// Sets custom risk haircuts for the RSU projections.
    pub const fn set_haircut_tiers(&mut self, tiers: HaircutTierTable) {
        self.haircut_tiers = Some(tiers);
    }

    /// Uses binary search to find the minimum monthly savings required to
    /// reach `target_cents` in exactly `target_months`.
    ///
    /// Returns `None` if the goal is unreachable even with a very high
    /// savings rate (e.g. if the target timeframe is 0 months and current
    /// net worth is below the target).
    #[must_use]
    pub fn find_required_savings(&self, target_cents: i64, target_months: u16) -> Option<i64> {
        if target_months == 0 {
            if self.initial_net_worth_cents >= target_cents {
                return Some(0);
            }
            return None;
        }

        // Binary search bounds:
        // Min savings is 0 (or technically negative, but let's assume non-negative savings).
        // Max savings is the entire difference divided by 1 month, just to set a high upper bound.
        let mut low = 0_i64;

        // Upper bound: if we saved the entire difference in one month.
        // We add a buffer of 1_000_000_000 to handle cases where we might need to save more than the difference (e.g. if target_cents < initial_net_worth_cents but we want to know if 0 is enough).
        let diff = target_cents.saturating_sub(self.initial_net_worth_cents);
        let mut high = diff.saturating_add(1_000_000_000);

        let mut best_savings: Option<i64> = None;

        // Perform binary search
        while low <= high {
            // Prevent overflow during midpoint calculation
            let mid = low + (high - low) / 2;

            let mut projector = NetWorthProjector::new(self.initial_net_worth_cents, mid);

            if let Some(tiers) = self.haircut_tiers {
                projector.set_haircut_tiers(tiers);
            }

            for vest in &self.upcoming_vests {
                // UpcomingVest only contains primitive copies, but we re-create it to avoid Clone bounds if not present.
                projector.add_upcoming_vest(UpcomingVest {
                    avg_close_price_cents: vest.avg_close_price_cents,
                    units: vest.units,
                    days_to_vest: vest.days_to_vest,
                });
            }

            // We don't strictly need to add the milestone to the projector for the timeline projection,
            // but we can to be thorough.
            projector.add_milestone_cents(target_cents);

            let (timeline, _) = projector.project_timeline(target_months);
            let final_nw = timeline
                .last()
                .map_or(self.initial_net_worth_cents, |m| m.net_worth_cents);

            if final_nw >= target_cents {
                // We reached the goal. Try to find a smaller savings amount.
                best_savings = Some(mid);
                high = mid.saturating_sub(1);
            } else {
                // We missed the goal. Need to save more.
                low = mid.saturating_add(1);
            }
        }

        best_savings
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_goal_seeker_zero_months() {
        let seeker = GoalSeeker::new(100_000_00);
        // Target is already met
        assert_eq!(seeker.find_required_savings(50_000_00, 0), Some(0));
        assert_eq!(seeker.find_required_savings(100_000_00, 0), Some(0));

        // Target is impossible in 0 months
        assert_eq!(seeker.find_required_savings(150_000_00, 0), None);
    }

    #[test]
    fn test_goal_seeker_no_vests() {
        // Start with $0. Want $120,000 in 12 months.
        // Needs exactly $10,000 / month.
        let seeker = GoalSeeker::new(0);

        let required = seeker
            .find_required_savings(120_000_00, 12)
            .expect("should find a solution");
        // Due to binary search exactness, it should find exactly 10,000.
        assert_eq!(required, 10_000_00);
    }

    #[test]
    fn test_goal_seeker_with_initial_net_worth() {
        // Start with $50,000. Want $150,000 in 10 months.
        // Difference is $100,000. Needs $10,000 / month.
        let seeker = GoalSeeker::new(50_000_00);

        let required = seeker
            .find_required_savings(150_000_00, 10)
            .expect("should find a solution");
        assert_eq!(required, 10_000_00);
    }

    #[test]
    fn test_goal_seeker_with_vests() {
        // Start with $0. Want $150,000 in 10 months.
        // Have a vest of $50,000 (after haircut) in month 5.
        let mut seeker = GoalSeeker::new(0);

        // Let's configure the vest so it has a known safe value.
        // Short tier is default 25% haircut.
        // Retained is 75%.
        // So gross needs to be $66,666.66... Let's just use custom tiers to make math easy.
        seeker.set_haircut_tiers(HaircutTierTable::new(0, 0, 0).unwrap()); // 0% haircut

        seeker.add_upcoming_vest(UpcomingVest {
            avg_close_price_cents: 100_00, // $100
            units: 500,                    // $50,000 total
            days_to_vest: 150,             // Month 5
        });

        // Total target: $150,000. Vest provides $50,000.
        // Need to save remaining $100,000 over 10 months.
        // Required savings: $10,000 / month.
        let required = seeker
            .find_required_savings(150_000_00, 10)
            .expect("should find a solution");
        assert_eq!(required, 10_000_00);
    }
}
