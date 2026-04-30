#![cfg(feature = "nova")]

//! Fee Drag Simulator
//!
//! A simulator to calculate the compounding loss of investment fees
//! (like Expense Ratios or AUM fees) over time, comparing a fee-laden
//! portfolio to a theoretical fee-free baseline.

/// The result of a fee drag simulation.
#[derive(Debug, Clone, PartialEq)]
pub struct FeeDragResult {
    /// The final balance of the fee-free portfolio in cents.
    pub baseline_balance_cents: i64,
    /// The final balance of the portfolio paying fees in cents.
    pub fee_balance_cents: i64,
    /// The total value lost to fees directly and lost compounding growth, in cents.
    pub total_lost_to_fees_cents: i64,
    /// The percentage of the final baseline balance that was lost to fees.
    pub percentage_lost: f64,
}

/// Simulates the impact of investment fees on portfolio growth over time.
#[derive(Debug, Clone)]
pub struct FeeDragSimulator {
    initial_balance_cents: i64,
    monthly_contribution_cents: i64,
    annual_return_pct: f64,
    annual_fee_pct: f64,
    years: u16,
}

impl FeeDragSimulator {
    /// Creates a new `FeeDragSimulator`.
    ///
    /// # Arguments
    /// * `initial_balance_cents` - Starting portfolio balance.
    /// * `monthly_contribution_cents` - Amount added to the portfolio every month.
    /// * `annual_return_pct` - Expected gross annual return (e.g., 7.0 for 7%).
    /// * `annual_fee_pct` - Annual fee percentage (e.g., 1.0 for a 1% AUM fee).
    /// * `years` - How many years to simulate.
    #[must_use]
    pub const fn new(
        initial_balance_cents: i64,
        monthly_contribution_cents: i64,
        annual_return_pct: f64,
        annual_fee_pct: f64,
        years: u16,
    ) -> Self {
        Self {
            initial_balance_cents,
            monthly_contribution_cents,
            annual_return_pct,
            annual_fee_pct,
            years,
        }
    }

    /// Simulates the growth of the portfolio over the specified years.
    /// Returns the final balance in cents.
    #[must_use]
    fn simulate(&self, effective_annual_return_pct: f64) -> i64 {
        let mut balance = self.initial_balance_cents;
        let months = self.years * 12;

        let monthly_return_rate = if effective_annual_return_pct > 0.0 {
            (1.0 + effective_annual_return_pct / 100.0).powf(1.0 / 12.0) - 1.0
        } else {
            0.0
        };

        for _ in 0..months {
            if balance > 0 {
                #[allow(clippy::cast_precision_loss)]
                let returns = (balance as f64) * monthly_return_rate;
                #[allow(clippy::cast_possible_truncation)]
                let returns_cents = returns.round() as i64;
                balance = balance.saturating_add(returns_cents);
            }
            balance = balance.saturating_add(self.monthly_contribution_cents);
        }

        balance
    }

    /// Calculates the impact of the fees compared to a fee-free baseline.
    #[must_use]
    pub fn calculate(&self) -> FeeDragResult {
        let baseline_balance = self.simulate(self.annual_return_pct);
        let fee_balance = self.simulate(self.annual_return_pct - self.annual_fee_pct);

        let total_lost = baseline_balance.saturating_sub(fee_balance);

        let percentage_lost = if baseline_balance > 0 {
            #[allow(clippy::cast_precision_loss)]
            let lost_f64 = total_lost as f64;
            #[allow(clippy::cast_precision_loss)]
            let baseline_f64 = baseline_balance as f64;
            (lost_f64 / baseline_f64) * 100.0
        } else {
            0.0
        };

        FeeDragResult {
            baseline_balance_cents: baseline_balance,
            fee_balance_cents: fee_balance,
            total_lost_to_fees_cents: total_lost,
            percentage_lost,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_fees() {
        let sim = FeeDragSimulator::new(
            1_000_000, // $10k initial
            100_000,   // $1k monthly
            7.0,       // 7% return
            0.0,       // 0% fees
            10,        // 10 years
        );

        let result = sim.calculate();

        assert_eq!(result.baseline_balance_cents, result.fee_balance_cents);
        assert_eq!(result.total_lost_to_fees_cents, 0);
        assert!((result.percentage_lost - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_high_fees() {
        let sim = FeeDragSimulator::new(
            10_000_000, // $100k initial
            0,          // no contributions
            7.0,        // 7% return
            2.0,        // 2% fee!
            30,         // 30 years
        );

        let result = sim.calculate();

        assert!(result.baseline_balance_cents > result.fee_balance_cents);
        assert!(result.total_lost_to_fees_cents > 0);

        // A 2% fee over 30 years on a 7% return should eat roughly 40%+ of the final value
        assert!(result.percentage_lost > 40.0);
    }

    #[test]
    fn test_negative_effective_return() {
        // Just checking it doesn't break, though returns < 0 are clamped to 0 inside simulate for simplicity
        let sim = FeeDragSimulator::new(
            1_000_000, 0, 2.0, 3.0, // 3% fee on 2% return = -1% effective return
            10,
        );

        let result = sim.calculate();
        // Since negative returns are effectively treated as 0% monthly return in this simple model
        // baseline grows, fee balance stays flat (or drops if we allowed negative).
        // Since we clamped negative to 0.0 in simulate(), it just stays flat.
        assert_eq!(result.fee_balance_cents, 1_000_000);
        assert!(result.baseline_balance_cents > 1_000_000);
    }
}
