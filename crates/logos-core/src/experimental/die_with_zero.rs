//! Die With Zero Simulator
//!
//! Standard FIRE models emphasize a "Safe Withdrawal Rate" designed to preserve capital forever.
//! The "Die With Zero" philosophy asks: What if you intentionally draw down your portfolio
//! to zero by your life expectancy?

/// Simulator for the "Die With Zero" strategy.
#[derive(Debug, Clone)]
pub struct DieWithZeroSimulator {
    current_age_years: u8,
    life_expectancy_years: u8,
    current_net_worth_cents: i64,
    annual_real_return_pct: f64,
}

impl DieWithZeroSimulator {
    #[must_use]
    pub const fn new(
        current_age_years: u8,
        life_expectancy_years: u8,
        current_net_worth_cents: i64,
        annual_real_return_pct: f64,
    ) -> Self {
        Self {
            current_age_years,
            life_expectancy_years,
            current_net_worth_cents,
            annual_real_return_pct,
        }
    }

    #[must_use]
    pub fn maximum_monthly_burn_cents(&self) -> i64 {
        if self.current_age_years >= self.life_expectancy_years {
            return 0;
        }
        if self.current_net_worth_cents <= 0 {
            return 0;
        }

        let months = u32::from(self.life_expectancy_years - self.current_age_years) * 12;

        if self.annual_real_return_pct <= 0.0 {
            return self.current_net_worth_cents / (i64::from(months));
        }

        let r = (self.annual_real_return_pct / 100.0) / 12.0;
        #[allow(clippy::cast_possible_wrap)]
        let one_plus_r_pow_n = (1.0 + r).powi(months as i32);

        #[allow(clippy::cast_precision_loss)]
        let pmt = (self.current_net_worth_cents as f64) * (r * one_plus_r_pow_n)
            / (one_plus_r_pow_n - 1.0);

        #[allow(clippy::cast_possible_truncation)]
        let pmt_cents = pmt as i64;

        pmt_cents
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_maximum_monthly_burn() {
        // Starting with $1,000,000 at age 60, expecting to die at 85 (25 years = 300 months)
        // With a 5% real return.
        let sim = DieWithZeroSimulator::new(60, 85, 100_000_000, 5.0);

        // PMT = $1M * (0.004166 * (1.004166)^300) / ((1.004166)^300 - 1)
        // Approximate expected value is $5,845 per month ($584,590 cents)
        let burn = sim.maximum_monthly_burn_cents();
        assert!(burn > 580_000 && burn < 590_000, "Burn was {burn}");
    }
}
