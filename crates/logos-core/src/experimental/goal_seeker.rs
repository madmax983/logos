use crate::domain::rsu::HaircutTierTable;
use crate::planning::fire::UpcomingVest;
use crate::planning::net_worth_projector::NetWorthProjector;

/// A simulator that calculates the required monthly savings (or allowable burn rate)
/// to achieve a specific target net worth by a specific target month.
///
/// It uses a binary search algorithm over the `NetWorthProjector` to find the optimal
/// monthly cash flow. If the target is easily achievable, it will output a negative
/// required savings (a "Safe Burn Rate").
#[derive(Debug, Clone)]
pub struct GoalSeeker {
    initial_net_worth_cents: i64,
    target_net_worth_cents: i64,
    target_months: u16,
    upcoming_vests: Vec<UpcomingVest>,
    haircut_tiers: HaircutTierTable,
}

impl GoalSeeker {
    /// Creates a new `GoalSeeker`.
    #[must_use]
    pub fn new(
        initial_net_worth_cents: i64,
        target_net_worth_cents: i64,
        target_months: u16,
    ) -> Self {
        Self {
            initial_net_worth_cents,
            target_net_worth_cents,
            target_months,
            upcoming_vests: Vec::new(),
            haircut_tiers: HaircutTierTable::default(),
        }
    }

    /// Sets the `HaircutTierTable` used to discount future RSU vests.
    pub const fn set_haircut_tiers(&mut self, tiers: HaircutTierTable) {
        self.haircut_tiers = tiers;
    }

    /// Registers an `UpcomingVest` to be included in the projection timeline.
    pub fn add_upcoming_vest(&mut self, vest: UpcomingVest) {
        self.upcoming_vests.push(vest);
    }

    /// Finds the required monthly savings in cents to hit the target.
    ///
    /// If the result is negative, it represents a "Safe Burn Rate" (how much you can
    /// safely lose or spend extra per month and still hit the target).
    ///
    /// The search is bounded between -$10M/mo and +$10M/mo by default, returning
    /// the closest achievable value.
    #[must_use]
    pub fn find_required_monthly_savings_cents(&self) -> i64 {
        if self.target_months == 0 {
            return 0;
        }

        let mut low: i64 = -1_000_000_000;
        let mut high: i64 = 1_000_000_000;
        let mut best_savings: i64 = high;

        while low <= high {
            let mid = low + (high - low) / 2;

            let mut projector = NetWorthProjector::new(self.initial_net_worth_cents, mid);
            projector.set_haircut_tiers(self.haircut_tiers);

            for vest in &self.upcoming_vests {
                projector.add_upcoming_vest(*vest);
            }

            let (timeline, _) = projector.project_timeline(self.target_months);
            let final_nw = timeline
                .last()
                .map_or(self.initial_net_worth_cents, |m| m.net_worth_cents);

            if final_nw >= self.target_net_worth_cents {
                best_savings = mid;
                high = mid - 1;
            } else {
                low = mid + 1;
            }
        }

        best_savings
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_positive_savings_required() {
        let seeker = GoalSeeker::new(1_000_000, 2_200_000, 12);
        let req = seeker.find_required_monthly_savings_cents();
        assert_eq!(req, 100_000);
    }

    #[test]
    fn test_safe_burn_rate() {
        let seeker = GoalSeeker::new(5_000_000, 2_600_000, 12);
        let req = seeker.find_required_monthly_savings_cents();
        assert_eq!(req, -200_000);
    }

    #[test]
    fn test_with_vests() {
        let mut seeker = GoalSeeker::new(0, 5_000_000, 5);
        seeker.set_haircut_tiers(HaircutTierTable::new(0, 0, 0).unwrap());

        seeker.add_upcoming_vest(UpcomingVest {
            avg_close_price_cents: 10_000,
            units: 250,
            days_to_vest: 80,
        });

        let req = seeker.find_required_monthly_savings_cents();
        assert_eq!(req, 500_000);
    }

    #[test]
    fn test_zero_months() {
        let seeker = GoalSeeker::new(100_000, 200_000, 0);
        let req = seeker.find_required_monthly_savings_cents();
        assert_eq!(req, 0);
    }
}
