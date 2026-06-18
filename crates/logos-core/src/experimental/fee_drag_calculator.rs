#![cfg(feature = "nova")]

//! Calculates the long-term drag of investment fees (expense ratios or AUM fees).

/// A calculator to demonstrate how much wealth is lost to investment fees over time.
#[derive(Debug, Clone, Copy)]
pub struct FeeDragCalculator {
    initial_balance_cents: i64,
    monthly_contribution_cents: i64,
    annual_return_pct: f64,
}

impl FeeDragCalculator {
    /// Creates a new `FeeDragCalculator`.
    #[must_use]
    pub const fn new(
        initial_balance_cents: i64,
        monthly_contribution_cents: i64,
        annual_return_pct: f64,
    ) -> Self {
        Self {
            initial_balance_cents,
            monthly_contribution_cents,
            annual_return_pct,
        }
    }

    /// Calculates the total cents lost to the expense ratio over the given number of years.
    #[must_use]
    #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
    pub fn calculate_drag(&self, years: u16, expense_ratio_pct: f64) -> i64 {
        let mut balance_no_fee = self.initial_balance_cents as f64;
        let mut balance_with_fee = self.initial_balance_cents as f64;

        let monthly_return = self.annual_return_pct / 1200.0;
        let monthly_return_with_fee = (self.annual_return_pct - expense_ratio_pct) / 1200.0;

        for _ in 0..(years * 12) {
            balance_no_fee *= 1.0 + monthly_return;
            balance_no_fee += self.monthly_contribution_cents as f64;

            balance_with_fee *= 1.0 + monthly_return_with_fee;
            balance_with_fee += self.monthly_contribution_cents as f64;
        }

        (balance_no_fee - balance_with_fee).round() as i64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_fee_drag() {
        let calc = FeeDragCalculator::new(10_000_000, 100_000, 7.0);
        let drag = calc.calculate_drag(30, 1.0);
        assert!(drag > 0);
        assert_eq!(drag, 42_484_818); // Lost to 1% fee over 30 years
    }
}
