//! Goal Fund Projector
//!
//! A simulator that tracks the growth of a specific financial bucket (e.g., a house down payment fund)
//! over time. It combines dedicated monthly savings with the fractional value of upcoming RSU vests,
//! strictly adhering to an `AllocationPolicy`.

use crate::domain::rsu::{AllocationPolicy, HaircutTierTable, forecast_value_cents};
use crate::error::DomainError;
use crate::planning::fire::UpcomingVest;

/// Represents a single month's snapshot in a goal fund projection timeline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectedFundMonth {
    /// The month index in the projection (0 is the current month).
    pub month_index: u16,
    /// The fund balance in cents at the end of this month.
    pub fund_balance_cents: i64,
    /// The portion of vested RSU value allocated to the fund this month, in cents.
    pub allocated_vest_cents: i64,
    /// The cash saved directly to the fund this month, in cents.
    pub saved_cents: i64,
}

/// Projects a specific fund's growth over time using dedicated savings and RSU allocations.
#[derive(Debug, Clone)]
pub struct GoalFundProjector {
    initial_balance_cents: i64,
    monthly_savings_cents: i64,
    upcoming_vests: Vec<UpcomingVest>,
    haircut_tiers: HaircutTierTable,
    allocation_policy: AllocationPolicy,
    target_goal_cents: i64,
}

impl GoalFundProjector {
    /// Creates a new `GoalFundProjector` to simulate the timeline for a specific financial goal.
    #[must_use]
    pub fn new(
        initial_balance_cents: i64,
        monthly_savings_cents: i64,
        allocation_policy: AllocationPolicy,
        target_goal_cents: i64,
    ) -> Self {
        Self {
            initial_balance_cents,
            monthly_savings_cents,
            upcoming_vests: Vec::new(),
            haircut_tiers: HaircutTierTable::conservative_defaults(),
            allocation_policy,
            target_goal_cents,
        }
    }

    /// Registers an upcoming vest to be included in the projection timeline.
    pub fn add_upcoming_vest(&mut self, vest: UpcomingVest) {
        self.upcoming_vests.push(vest);
    }

    /// Simulates the fund balance month-by-month for `months` iterations.
    ///
    /// Returns a tuple containing:
    /// 1. A timeline of [`ProjectedFundMonth`] snapshots.
    /// 2. An `Option<u16>` indicating the exact month the target goal was reached (if any).
    ///
    /// # Errors
    /// Returns `DomainError::AmountOverflow` if the internal fund balance or
    /// allocated vest amounts overflow `i64` bounds.
    pub fn project_timeline(
        &self,
        months: u16,
    ) -> Result<(Vec<ProjectedFundMonth>, Option<u16>), DomainError> {
        let mut timeline = Vec::with_capacity(usize::from(months));
        let mut goal_reached_month = None;
        let mut current_balance = self.initial_balance_cents;

        for month_index in 1..=months {
            let mut allocated_this_month = 0_i64;

            let month_start_days = (month_index - 1) * 30;
            let month_end_days = month_index * 30;

            for vest in &self.upcoming_vests {
                if vest.days_to_vest > month_start_days && vest.days_to_vest <= month_end_days {
                    let safe_value = forecast_value_cents(
                        vest.avg_close_price_cents,
                        vest.units,
                        vest.days_to_vest,
                        &self.haircut_tiers,
                    );
                    // Apply the allocation policy to the vested value.
                    let allocated_value = safe_value
                        .checked_mul(i64::from(self.allocation_policy.goals_pct()))
                        .map(|v| v / 100)
                        .ok_or(DomainError::AmountOverflow)?;

                    allocated_this_month = allocated_this_month
                        .checked_add(allocated_value)
                        .ok_or(DomainError::AmountOverflow)?;
                }
            }

            current_balance = current_balance
                .checked_add(self.monthly_savings_cents)
                .ok_or(DomainError::AmountOverflow)?
                .checked_add(allocated_this_month)
                .ok_or(DomainError::AmountOverflow)?;

            if goal_reached_month.is_none() && current_balance >= self.target_goal_cents {
                goal_reached_month = Some(month_index);
            }

            timeline.push(ProjectedFundMonth {
                month_index,
                fund_balance_cents: current_balance,
                allocated_vest_cents: allocated_this_month,
                saved_cents: self.monthly_savings_cents,
            });
        }

        Ok((timeline, goal_reached_month))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_fund_timeline() {
        // 50% tax, 10% smoothing, 40% goals, 0% discretionary
        let policy = AllocationPolicy::new(50, 10, 40, 0).unwrap();
        // Goal: $20,000 (2_000_000 cents)
        // Initial: $5,000 (500_000 cents)
        // Monthly savings to this fund: $1,000 (100_000 cents)
        let mut projector = GoalFundProjector::new(500_000, 100_000, policy, 2_000_000);

        // Add a vest in month 2 (45 days)
        // 100 units at $100 ($10,000 gross).
        // Medium tier haircut = 40% off, so $6,000 safe value.
        // 40% goals policy of $6,000 = $2,400 allocated (240_000 cents).
        projector.add_upcoming_vest(UpcomingVest {
            avg_close_price_cents: 10_000,
            units: 100,
            days_to_vest: 45,
        });

        let (timeline, goal_month) = projector.project_timeline(5).unwrap();

        assert_eq!(timeline.len(), 5);

        // Month 1
        assert_eq!(timeline[0].fund_balance_cents, 600_000);
        assert_eq!(timeline[0].allocated_vest_cents, 0);

        // Month 2
        // 600_000 + 100_000 (savings) + 240_000 (vest) = 940_000
        assert_eq!(timeline[1].fund_balance_cents, 940_000);
        assert_eq!(timeline[1].allocated_vest_cents, 240_000);

        // Month 3: 940_000 + 100_000 = 1_040_000
        assert_eq!(timeline[2].fund_balance_cents, 1_040_000);

        assert_eq!(goal_month, None); // Target not reached in 5 months
    }

    #[test]
    fn test_goal_reached() {
        let policy = AllocationPolicy::new(50, 10, 40, 0).unwrap();
        let projector = GoalFundProjector::new(1_000_000, 500_000, policy, 2_000_000);

        let (timeline, goal_month) = projector.project_timeline(3).unwrap();

        // M1: 1.5M, M2: 2.0M
        assert_eq!(goal_month, Some(2));
        assert_eq!(timeline[1].fund_balance_cents, 2_000_000);
    }

    #[test]
    fn test_amount_overflow() {
        let policy = AllocationPolicy::new(50, 10, 40, 0).unwrap();
        let projector = GoalFundProjector::new(i64::MAX, 100_000, policy, 2_000_000);

        let result = projector.project_timeline(1);
        assert_eq!(result, Err(DomainError::AmountOverflow));
    }
}
