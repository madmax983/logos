//! Asset Depreciation Simulator
//!
//! A module to project the future value of physical assets (like vehicles or hardware)
//! using standard accounting depreciation schedules.

/// The schedule used to calculate asset depreciation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DepreciationSchedule {
    /// Loses a fixed percentage of its ORIGINAL value each year.
    Linear {
        useful_life_years: u16,
        salvage_value_cents: i64,
    },
    /// Loses a fixed percentage of its CURRENT value each year.
    DecliningBalance {
        depreciation_rate_pct: u8,
        salvage_value_cents: i64,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetDepreciationSimulator {
    initial_value_cents: i64,
    schedule: DepreciationSchedule,
}

impl AssetDepreciationSimulator {
    #[must_use]
    pub const fn new(initial_value_cents: i64, schedule: DepreciationSchedule) -> Self {
        Self {
            initial_value_cents,
            schedule,
        }
    }

    /// Projects the value of the asset after a given number of years.
    #[must_use]
    pub fn value_after_years(&self, years: u16) -> i64 {
        match self.schedule {
            DepreciationSchedule::Linear {
                useful_life_years,
                salvage_value_cents,
            } => {
                if years >= useful_life_years {
                    return salvage_value_cents;
                }

                let total_depreciable =
                    self.initial_value_cents.saturating_sub(salvage_value_cents);
                // Avoid division by zero if useful_life_years is 0 somehow
                if useful_life_years == 0 {
                    return salvage_value_cents;
                }

                let yearly_depreciation = total_depreciable / i64::from(useful_life_years);
                let total_depreciated = yearly_depreciation.saturating_mul(i64::from(years));

                self.initial_value_cents.saturating_sub(total_depreciated)
            }
            DepreciationSchedule::DecliningBalance {
                depreciation_rate_pct,
                salvage_value_cents,
            } => {
                let mut current_value = self.initial_value_cents;

                for _ in 0..years {
                    if current_value <= salvage_value_cents {
                        break;
                    }

                    let depreciation_amount =
                        current_value.saturating_mul(i64::from(depreciation_rate_pct)) / 100;
                    current_value = current_value.saturating_sub(depreciation_amount);
                }

                std::cmp::max(current_value, salvage_value_cents)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_depreciation() {
        let schedule = DepreciationSchedule::Linear {
            useful_life_years: 5,
            salvage_value_cents: 200_000,
        };
        let simulator = AssetDepreciationSimulator::new(1_000_000, schedule);

        assert_eq!(simulator.value_after_years(0), 1_000_000);
        assert_eq!(simulator.value_after_years(1), 840_000);
        assert_eq!(simulator.value_after_years(3), 520_000);
        assert_eq!(simulator.value_after_years(5), 200_000);
        assert_eq!(simulator.value_after_years(10), 200_000);
    }

    #[test]
    fn test_declining_balance_depreciation() {
        let schedule = DepreciationSchedule::DecliningBalance {
            depreciation_rate_pct: 20,
            salvage_value_cents: 200_000,
        };
        let simulator = AssetDepreciationSimulator::new(1_000_000, schedule);

        assert_eq!(simulator.value_after_years(0), 1_000_000);
        assert_eq!(simulator.value_after_years(1), 800_000);
        assert_eq!(simulator.value_after_years(2), 640_000);
        assert_eq!(simulator.value_after_years(3), 512_000);
        assert_eq!(simulator.value_after_years(50), 200_000);
    }

    #[test]
    fn test_linear_depreciation_zero_useful_life() {
        let schedule = DepreciationSchedule::Linear {
            useful_life_years: 0,
            salvage_value_cents: 200_000,
        };
        let simulator = AssetDepreciationSimulator::new(1_000_000, schedule);

        assert_eq!(simulator.value_after_years(0), 200_000);
        assert_eq!(simulator.value_after_years(1), 200_000);
    }
}
