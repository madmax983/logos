//! Restricted Stock Unit (RSU) forecasting and allocation planning.
//!
//! This module provides structures and functions to model unvested equity.
//! Because equity prices are volatile, `logos` uses a "haircut" (discount) approach
//! to forecast the safe spendable value of future vests.

use crate::error::DomainError;

/// Defines discount percentages applied to unvested equity based on time horizon.
///
/// A "haircut" reduces the projected value of a vest to account for market risk.
/// Vests further in the future receive larger haircuts (larger discounts).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HaircutTierTable {
    short: u8,
    medium: u8,
    long: u8,
}

impl Default for HaircutTierTable {
    /// Creates a table with default conservative haircut tiers.
    ///
    /// * **Short** (< 30 days): 25% discount
    /// * **Medium** (<= 90 days): 40% discount
    /// * **Long** (> 90 days): 55% discount
    fn default() -> Self {
        Self {
            short: 25,
            medium: 40,
            long: 55,
        }
    }
}

impl HaircutTierTable {
    /// Creates a custom haircut tier table.
    ///
    /// ## Examples
    ///
    /// ```
    /// use logos_core::domain::rsu::HaircutTierTable;
    ///
    /// let tiers = HaircutTierTable::new(20, 30, 50).expect("valid tiers");
    /// assert_eq!(tiers.haircut_for_days(15), 20); // short (< 30 days)
    /// assert_eq!(tiers.haircut_for_days(45), 30); // medium (<= 90 days)
    /// assert_eq!(tiers.haircut_for_days(120), 50); // long (> 90 days)
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if any percentage is greater than `100` or if
    /// tiers are not non-decreasing by time horizon (`short <= medium <= long`).
    pub fn new(short: u8, medium: u8, long: u8) -> Result<Self, DomainError> {
        for (tier, percentage) in [("short", short), ("medium", medium), ("long", long)] {
            if percentage > 100 {
                return Err(DomainError::InvalidHaircutPercentage { tier, percentage });
            }
        }

        if short > medium || medium > long {
            return Err(DomainError::InvalidHaircutOrdering {
                short,
                medium,
                long,
            });
        }

        Ok(Self {
            short,
            medium,
            long,
        })
    }

    /// Creates a table with default conservative haircut tiers.
    #[must_use]
    pub fn conservative_defaults() -> Self {
        Self::default()
    }

    /// Returns the haircut percentage for a given number of days to vest.
    ///
    /// ## Examples
    ///
    /// ```
    /// use logos_core::domain::rsu::HaircutTierTable;
    ///
    /// let tiers = HaircutTierTable::default();
    /// assert_eq!(tiers.haircut_for_days(15), 25);
    /// assert_eq!(tiers.haircut_for_days(60), 40);
    /// assert_eq!(tiers.haircut_for_days(120), 55);
    /// ```
    #[must_use]
    pub const fn haircut_for_days(&self, days_to_vest: u16) -> u8 {
        if days_to_vest < 30 {
            self.short
        } else if days_to_vest <= 90 {
            self.medium
        } else {
            self.long
        }
    }
}

/// A policy defining how the after-tax value of an RSU vest will be distributed.
///
/// The policy divides the projected value into four categories:
/// tax reserves, income smoothing buffers, specific financial goals,
/// and discretionary spending.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AllocationPolicy {
    tax_reserve: u8,
    smoothing_buffer: u8,
    goals: u8,
    discretionary: u8,
}

impl AllocationPolicy {
    /// Creates an allocation policy that must total exactly 100%.
    ///
    /// ## Examples
    ///
    /// ```
    /// use logos_core::domain::rsu::AllocationPolicy;
    ///
    /// // 40% tax, 20% smoothing, 30% goals, 10% discretionary = 100%
    /// let policy = AllocationPolicy::new(40, 20, 30, 10).expect("valid policy");
    /// assert_eq!(policy.tax_reserve_pct(), 40);
    /// assert_eq!(policy.goals_pct(), 30);
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error when the percentage total is not exactly `100`.
    pub fn new(
        tax_reserve_pct: u8,
        smoothing_buffer_pct: u8,
        goals_pct: u8,
        discretionary_pct: u8,
    ) -> Result<Self, DomainError> {
        let total = u16::from(tax_reserve_pct)
            + u16::from(smoothing_buffer_pct)
            + u16::from(goals_pct)
            + u16::from(discretionary_pct);

        if total != 100 {
            return Err(DomainError::InvalidAllocationTotal { total });
        }

        Ok(Self {
            tax_reserve: tax_reserve_pct,
            smoothing_buffer: smoothing_buffer_pct,
            goals: goals_pct,
            discretionary: discretionary_pct,
        })
    }

    #[must_use]
    pub const fn tax_reserve_pct(&self) -> u8 {
        self.tax_reserve
    }

    #[must_use]
    pub const fn smoothing_buffer_pct(&self) -> u8 {
        self.smoothing_buffer
    }

    #[must_use]
    pub const fn goals_pct(&self) -> u8 {
        self.goals
    }

    #[must_use]
    pub const fn discretionary_pct(&self) -> u8 {
        self.discretionary
    }
}

/// Projects the safe value of an upcoming RSU vest in cents.
///
/// This function calculates the gross value (average price * units) and then
/// applies the appropriate discount (haircut) based on the time horizon.
/// Invalid input (negative price) and arithmetic overflow fail closed to `0`.
///
/// ## Examples
///
/// ```
/// use logos_core::domain::rsu::{HaircutTierTable, forecast_value_cents};
///
/// let tiers = HaircutTierTable::default();
/// // 100 units at $10.00 (1000 cents) vesting in 15 days (short tier, 25% haircut).
/// // Gross = $1000. Retained = 75%. Result = $750 (75000 cents).
/// let safe_value = forecast_value_cents(1000, 100, 15, &tiers);
/// assert_eq!(safe_value, 75000);
/// ```
#[must_use]
pub fn forecast_value_cents(
    avg_close_price_cents: i64,
    units: u32,
    days_to_vest: u16,
    tiers: &HaircutTierTable,
) -> i64 {
    if avg_close_price_cents < 0 {
        return 0;
    }

    let haircut_pct = i64::from(tiers.haircut_for_days(days_to_vest));
    let retained_pct = 100_i64.saturating_sub(haircut_pct);
    let Some(gross) = avg_close_price_cents.checked_mul(i64::from(units)) else {
        return 0;
    };

    gross
        .checked_mul(retained_pct)
        .map_or(0, |adjusted| adjusted / 100)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_create_valid_custom_haircut_tiers() {
        let tiers = HaircutTierTable::new(20, 40, 60).expect("valid custom haircut tiers");
        assert_eq!(tiers.haircut_for_days(1), 20);
        assert_eq!(tiers.haircut_for_days(45), 40);
        assert_eq!(tiers.haircut_for_days(180), 60);
    }

    #[test]
    fn should_reject_haircut_percentage_above_100() {
        assert_eq!(
            HaircutTierTable::new(20, 40, 101),
            Err(DomainError::InvalidHaircutPercentage {
                tier: "long",
                percentage: 101
            })
        );
    }

    #[test]
    fn should_reject_non_monotonic_haircut_tiers() {
        assert_eq!(
            HaircutTierTable::new(50, 40, 60),
            Err(DomainError::InvalidHaircutOrdering {
                short: 50,
                medium: 40,
                long: 60
            })
        );
    }

    #[test]
    fn should_forecast_expected_safe_value() {
        let tiers = HaircutTierTable::default();
        assert_eq!(forecast_value_cents(1_000, 100, 15, &tiers), 75_000);
    }

    #[test]
    fn should_return_zero_for_negative_price_input() {
        let tiers = HaircutTierTable::default();
        assert_eq!(forecast_value_cents(-1_000, 100, 15, &tiers), 0);
    }

    #[test]
    fn should_return_zero_when_forecast_overflows() {
        let tiers = HaircutTierTable::default();
        assert_eq!(forecast_value_cents(i64::MAX, 2, 15, &tiers), 0);
    }
}
