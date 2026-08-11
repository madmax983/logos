//! Die With Zero Simulator
//!
//! A calculator based on the philosophy of maximizing life experiences and
//! deliberately spending down wealth to hit exactly $0 by the end of your life expectancy.

/// Simulates the exact annual withdrawal amount needed to exhaust the portfolio
/// by a target age, assuming constant real returns.
#[derive(Debug, Clone)]
pub struct DieWithZeroSimulator {
    current_age: u8,
    life_expectancy: u8,
    portfolio_cents: i64,
    annual_return_pct: f64,
    annual_inflation_pct: f64,
}

impl DieWithZeroSimulator {
    /// Creates a new `DieWithZeroSimulator`.
    #[must_use]
    pub const fn new(
        current_age: u8,
        life_expectancy: u8,
        portfolio_cents: i64,
        annual_return_pct: f64,
        annual_inflation_pct: f64,
    ) -> Self {
        Self {
            current_age,
            life_expectancy,
            portfolio_cents,
            annual_return_pct,
            annual_inflation_pct,
        }
    }

    /// Calculates the maximum annual spend (in today's dollars) that will exactly
    /// exhaust the portfolio at the target age.
    ///
    /// Uses the present value of an annuity formula.
    #[must_use]
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_precision_loss,
        clippy::cast_sign_loss
    )]
    pub fn calculate_max_annual_spend_cents(&self) -> i64 {
        if self.current_age >= self.life_expectancy || self.portfolio_cents <= 0 {
            return self.portfolio_cents.max(0);
        }

        let years = f64::from(self.life_expectancy - self.current_age);

        // Calculate the real return rate: (1 + nominal) / (1 + inflation) - 1
        let real_return_rate = (1.0 + self.annual_return_pct / 100.0)
            / (1.0 + self.annual_inflation_pct / 100.0)
            - 1.0;

        if real_return_rate.abs() < f64::EPSILON {
            // Simple division if real return is exactly 0
            return (self.portfolio_cents as f64 / years) as i64;
        }

        // PMT formula for present value of annuity
        // PMT = PV * r / (1 - (1 + r)^-n)
        let pv = self.portfolio_cents as f64;
        let pmt = pv * real_return_rate / (1.0 - (1.0 + real_return_rate).powf(-years));

        pmt.max(0.0) as i64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_die_with_zero_basic() {
        // 60 years old, lives to 90 (30 years)
        // $1,000,000 portfolio (100_000_000 cents)
        // 7% return, 3% inflation (approx 3.88% real return)
        let sim = DieWithZeroSimulator::new(60, 90, 100_000_000, 7.0, 3.0);
        let max_spend = sim.calculate_max_annual_spend_cents();

        // Let's verify it's around $57,000
        assert!(max_spend > 5_000_000 && max_spend < 6_000_000);
    }

    #[test]
    fn test_die_with_zero_zero_real_return() {
        // 60 years old, lives to 80 (20 years)
        // $1,000,000 portfolio
        // 3% return, 3% inflation (0% real return)
        let sim = DieWithZeroSimulator::new(60, 80, 100_000_000, 3.0, 3.0);
        let max_spend = sim.calculate_max_annual_spend_cents();

        // Should be exactly 1_000_000 / 20 = 50_000 ($50,000/yr)
        assert_eq!(max_spend, 5_000_000);
    }

    #[test]
    fn test_die_with_zero_negative_real_return() {
        // 60 years old, lives to 70 (10 years)
        // $1,000,000 portfolio
        // 2% return, 5% inflation (-2.85% real return)
        let sim = DieWithZeroSimulator::new(60, 70, 100_000_000, 2.0, 5.0);
        let max_spend = sim.calculate_max_annual_spend_cents();

        // It should be less than 100,000 (since money loses value)
        assert!(max_spend < 10_000_000);
        assert!(max_spend > 8_000_000);
    }

    #[test]
    fn test_die_with_zero_already_dead() {
        let sim = DieWithZeroSimulator::new(90, 85, 100_000_000, 5.0, 2.0);
        assert_eq!(sim.calculate_max_annual_spend_cents(), 100_000_000);
    }
}
