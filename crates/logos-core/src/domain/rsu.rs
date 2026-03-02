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

impl HaircutTierTable {
    /// Creates a table with default conservative haircut tiers.
    ///
    /// * **Short** (< 30 days): 25% discount
    /// * **Medium** (<= 90 days): 40% discount
    /// * **Long** (> 90 days): 55% discount
    #[must_use]
    pub const fn conservative_defaults() -> Self {
        Self {
            short: 25,
            medium: 40,
            long: 55,
        }
    }

    /// Returns the haircut percentage for a given number of days to vest.
    ///
    /// ## Examples
    ///
    /// ```
    /// use logos_core::domain::rsu::HaircutTierTable;
    ///
    /// let tiers = HaircutTierTable::conservative_defaults();
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
///
/// ## Examples
///
/// ```
/// use logos_core::domain::rsu::{HaircutTierTable, forecast_value_cents};
///
/// let tiers = HaircutTierTable::conservative_defaults();
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
    let haircut_pct = i64::from(tiers.haircut_for_days(days_to_vest));
    let retained_pct = 100_i64.saturating_sub(haircut_pct);
    let gross = avg_close_price_cents.saturating_mul(i64::from(units));
    gross.saturating_mul(retained_pct) / 100
}
