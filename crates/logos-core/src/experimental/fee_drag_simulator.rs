#![cfg(feature = "nova")]

//! Investment Fee Drag Simulator
//!
//! Visualizes the long-term impact of management fees and expense ratios
//! on compound growth. Even a seemingly small 1% fee can consume a massive
//! portion of potential wealth over decades.

/// Represents the result of a fee drag simulation.
#[derive(Debug, Clone, PartialEq)]
pub struct FeeDragResult {
    /// Final portfolio value if there were no fees (in cents).
    pub no_fee_final_cents: i64,
    /// Final portfolio value after fees are deducted (in cents).
    pub fee_final_cents: i64,
    /// Total wealth lost to fees and lost compounding (in cents).
    pub total_lost_cents: i64,
    /// The percentage of potential wealth that was lost (0.0 to 100.0).
    pub wealth_lost_pct: f64,
}

/// A simulator to calculate the cost of investment fees over time.
#[derive(Debug, Clone)]
pub struct FeeDragSimulator {
    initial_investment_cents: i64,
    monthly_contribution_cents: i64,
    annual_return_pct: f64,
    annual_fee_pct: f64,
    years: u8,
}

impl FeeDragSimulator {
    /// Creates a new `FeeDragSimulator`.
    ///
    /// # Arguments
    /// * `initial_investment_cents` - Starting portfolio balance.
    /// * `monthly_contribution_cents` - Amount added to the portfolio each month.
    /// * `annual_return_pct` - Expected annual return before fees (e.g., 7.0 for 7%).
    /// * `annual_fee_pct` - Annual management fee or expense ratio (e.g., 1.0 for 1%).
    /// * `years` - Number of years to simulate compounding.
    #[must_use]
    pub const fn new(
        initial_investment_cents: i64,
        monthly_contribution_cents: i64,
        annual_return_pct: f64,
        annual_fee_pct: f64,
        years: u8,
    ) -> Self {
        Self {
            initial_investment_cents,
            monthly_contribution_cents,
            annual_return_pct,
            annual_fee_pct,
            years,
        }
    }

    /// Calculates the impact of the fees over the given timeframe.
    #[must_use]
    pub fn calculate(&self) -> FeeDragResult {
        let no_fee_final = Self::simulate(
            self.initial_investment_cents,
            self.monthly_contribution_cents,
            self.annual_return_pct,
            0.0,
            self.years,
        );

        let fee_final = Self::simulate(
            self.initial_investment_cents,
            self.monthly_contribution_cents,
            self.annual_return_pct,
            self.annual_fee_pct,
            self.years,
        );

        let total_lost = no_fee_final.saturating_sub(fee_final).max(0);

        #[allow(clippy::cast_precision_loss)]
        let wealth_lost_pct = if no_fee_final > 0 {
            (total_lost as f64 / no_fee_final as f64) * 100.0
        } else {
            0.0
        };

        FeeDragResult {
            no_fee_final_cents: no_fee_final,
            fee_final_cents: fee_final,
            total_lost_cents: total_lost,
            wealth_lost_pct,
        }
    }

    fn simulate(
        initial: i64,
        monthly_contrib: i64,
        annual_return: f64,
        annual_fee: f64,
        years: u8,
    ) -> i64 {
        let months = u32::from(years) * 12;
        let monthly_return_rate = annual_return / 100.0 / 12.0;
        let monthly_fee_rate = annual_fee / 100.0 / 12.0;

        #[allow(clippy::cast_precision_loss)]
        let mut balance = initial as f64;

        #[allow(clippy::cast_precision_loss)]
        let contrib = monthly_contrib as f64;

        for _ in 0..months {
            balance += contrib;
            balance *= 1.0 + monthly_return_rate;
            balance *= 1.0 - monthly_fee_rate;
        }

        #[allow(clippy::cast_possible_truncation)]
        let result = balance.round() as i64;
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_fees() {
        let sim = FeeDragSimulator::new(1_000_000, 100_000, 7.0, 0.0, 10);
        let result = sim.calculate();

        assert_eq!(result.no_fee_final_cents, result.fee_final_cents);
        assert_eq!(result.total_lost_cents, 0);
        assert!(result.wealth_lost_pct.abs() < f64::EPSILON);
    }

    #[test]
    fn test_one_percent_fee_impact() {
        // $100k initial, $1k/mo, 8% return, 1% fee, 30 years
        let sim = FeeDragSimulator::new(10_000_000, 100_000, 8.0, 1.0, 30);
        let result = sim.calculate();

        assert!(result.fee_final_cents < result.no_fee_final_cents);
        assert!(result.total_lost_cents > 0);

        // A 1% fee over 30 years usually eats 20-25% of potential wealth
        assert!(result.wealth_lost_pct > 20.0);
        assert!(result.wealth_lost_pct < 30.0);
    }

    #[test]
    fn test_no_growth_no_fee() {
        let sim = FeeDragSimulator::new(1_000_000, 100_000, 0.0, 0.0, 1);
        let result = sim.calculate();

        // $10k + $1k * 12 = $22k
        assert_eq!(result.no_fee_final_cents, 2_200_000);
        assert_eq!(result.fee_final_cents, 2_200_000);
    }
}
