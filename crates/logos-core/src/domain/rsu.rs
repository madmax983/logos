use crate::error::DomainError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HaircutTierTable {
    short: u8,
    medium: u8,
    long: u8,
}

impl HaircutTierTable {
    #[must_use]
    pub const fn conservative_defaults() -> Self {
        Self {
            short: 25,
            medium: 40,
            long: 55,
        }
    }

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
