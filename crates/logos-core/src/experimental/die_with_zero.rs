//! Die With Zero Simulator
//!
//! While standard FIRE models optimize for perpetual wealth preservation, the
//! "Die With Zero" philosophy optimizes for maximizing life experiences by
//! spending down your wealth so that you arrive at exactly $0 at the end of
//! your life expectancy.

/// Represents the simulation result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DieWithZeroResult {
    /// The maximum monthly amount you can safely spend to reach exactly zero by the end.
    pub max_monthly_spend_cents: i64,
    /// The total amount of money you will spend over your remaining lifetime.
    pub total_lifetime_spend_cents: i64,
}

/// A simulator to calculate optimal drawdown for a Die With Zero strategy.
#[derive(Debug, Clone)]
pub struct DieWithZeroSimulator {
    current_net_worth_cents: i64,
    current_age_years: u8,
    life_expectancy_years: u8,
    annual_return_pct: f64,
}

impl DieWithZeroSimulator {
    /// Creates a new `DieWithZeroSimulator`.
    ///
    /// # Arguments
    /// * `current_net_worth_cents` - Total available liquid assets in cents.
    /// * `current_age_years` - Your current age.
    /// * `life_expectancy_years` - The age you expect to live until.
    /// * `annual_return_pct` - Expected annual real return rate (e.g., 5.0 for 5%).
    #[must_use]
    pub const fn new(
        current_net_worth_cents: i64,
        current_age_years: u8,
        life_expectancy_years: u8,
        annual_return_pct: f64,
    ) -> Self {
        Self {
            current_net_worth_cents,
            current_age_years,
            life_expectancy_years,
            annual_return_pct,
        }
    }

    /// Calculates the maximum monthly spend.
    #[must_use]
    pub fn calculate(&self) -> DieWithZeroResult {
        if self.current_net_worth_cents <= 0 {
            return DieWithZeroResult {
                max_monthly_spend_cents: 0,
                total_lifetime_spend_cents: 0,
            };
        }

        if self.current_age_years >= self.life_expectancy_years {
            return DieWithZeroResult {
                max_monthly_spend_cents: self.current_net_worth_cents,
                total_lifetime_spend_cents: self.current_net_worth_cents,
            };
        }

        let months_remaining = u32::from(self.life_expectancy_years - self.current_age_years) * 12;

        let monthly_rate = if self.annual_return_pct > 0.0 {
            (1.0 + self.annual_return_pct / 100.0).powf(1.0 / 12.0) - 1.0
        } else {
            0.0
        };

        #[allow(clippy::cast_precision_loss)]
        let pv = self.current_net_worth_cents as f64;

        let pmt = if monthly_rate > 0.0 {
            #[allow(clippy::cast_possible_wrap)]
            let factor = 1.0 - (1.0 + monthly_rate).powi(-(months_remaining as i32));
            if factor == 0.0 {
                pv / f64::from(months_remaining)
            } else {
                (pv * monthly_rate) / factor
            }
        } else {
            pv / f64::from(months_remaining)
        };

        #[allow(clippy::cast_possible_truncation)]
        let max_monthly_spend_cents = pmt.round() as i64;

        let total_lifetime_spend_cents =
            max_monthly_spend_cents.saturating_mul(i64::from(months_remaining));

        DieWithZeroResult {
            max_monthly_spend_cents,
            total_lifetime_spend_cents,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_net_worth() {
        let sim = DieWithZeroSimulator::new(0, 40, 80, 5.0);
        let res = sim.calculate();
        assert_eq!(res.max_monthly_spend_cents, 0);
        assert_eq!(res.total_lifetime_spend_cents, 0);
    }

    #[test]
    fn test_already_at_life_expectancy() {
        let sim = DieWithZeroSimulator::new(10_000_000, 80, 80, 5.0);
        let res = sim.calculate();
        assert_eq!(res.max_monthly_spend_cents, 10_000_000);
        assert_eq!(res.total_lifetime_spend_cents, 10_000_000);
    }

    #[test]
    fn test_zero_growth() {
        let sim = DieWithZeroSimulator::new(120_000_000, 70, 80, 0.0);
        let res = sim.calculate();
        assert_eq!(res.max_monthly_spend_cents, 1_000_000);
        assert_eq!(res.total_lifetime_spend_cents, 120_000_000);
    }

    #[test]
    fn test_with_growth() {
        let sim = DieWithZeroSimulator::new(100_000_000, 50, 80, 5.0);
        let res = sim.calculate();
        assert!(res.max_monthly_spend_cents > 277_700);
        assert!(res.max_monthly_spend_cents < 600_000);
        assert!(res.total_lifetime_spend_cents > 100_000_000);
    }
}
