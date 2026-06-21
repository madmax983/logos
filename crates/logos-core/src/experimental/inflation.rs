//! Inflation simulation and purchasing power projection.
//!
//! A classic problem in long-term financial planning (like FIRE) is that $1 million
//! today does not equal $1 million in 20 years. This module provides a projector
//! to easily convert between present purchasing power and future nominal costs based
//! on expected inflation rates.

/// A projector to calculate the effects of inflation over time.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InflationProjector {
    /// The annual inflation rate as a percentage (e.g., 3.0 for 3%).
    pub annual_inflation_rate_pct: f64,
}

impl InflationProjector {
    /// Creates a new `InflationProjector` with the specified annual inflation rate.
    ///
    /// ## Examples
    ///
    /// ```
    /// use logos_core::InflationProjector;
    ///
    /// let projector = InflationProjector::new(3.0);
    /// assert_eq!(projector.annual_inflation_rate_pct, 3.0);
    /// ```
    #[must_use]
    pub const fn new(annual_inflation_rate_pct: f64) -> Self {
        Self {
            annual_inflation_rate_pct,
        }
    }

    /// Calculates the future nominal cost of an item that costs `present_cost_cents` today.
    ///
    /// Uses the compound interest formula: FV = PV * (1 + r)^n
    ///
    /// ## Examples
    ///
    /// ```
    /// use logos_core::InflationProjector;
    ///
    /// let projector = InflationProjector::new(3.0);
    /// // An item costing $10.00 today will cost ~$10.30 in 1 year at 3% inflation.
    /// let fv = projector.future_nominal_cost_cents(1000, 1);
    /// assert_eq!(fv, 1030);
    /// ```
    #[must_use]
    pub fn future_nominal_cost_cents(&self, present_cost_cents: i64, years: u16) -> i64 {
        if self.annual_inflation_rate_pct == 0.0 || years == 0 {
            return present_cost_cents;
        }

        let rate = self.annual_inflation_rate_pct / 100.0;
        let factor = (1.0 + rate).powi(i32::from(years));

        #[allow(clippy::cast_precision_loss)]
        let present_f64 = present_cost_cents as f64;
        let future_f64 = present_f64 * factor;

        #[allow(clippy::cast_possible_truncation)]
        let result = future_f64.round() as i64;
        result
    }

    /// Calculates the present purchasing power of a future nominal amount.
    ///
    /// Uses the present value discounting formula: PV = FV / (1 + r)^n
    ///
    /// ## Examples
    ///
    /// ```
    /// use logos_core::InflationProjector;
    ///
    /// let projector = InflationProjector::new(3.0);
    /// // $10.30 in 1 year at 3% inflation has the purchasing power of $10.00 today.
    /// let pv = projector.present_purchasing_power_cents(1030, 1);
    /// assert_eq!(pv, 1000);
    /// ```
    #[must_use]
    pub fn present_purchasing_power_cents(&self, future_nominal_cents: i64, years: u16) -> i64 {
        if self.annual_inflation_rate_pct == 0.0 || years == 0 {
            return future_nominal_cents;
        }

        let rate = self.annual_inflation_rate_pct / 100.0;
        let factor = (1.0 + rate).powi(i32::from(years));

        #[allow(clippy::cast_precision_loss)]
        let future_f64 = future_nominal_cents as f64;
        let present_f64 = future_f64 / factor;

        #[allow(clippy::cast_possible_truncation)]
        let result = present_f64.round() as i64;
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_inflation() {
        let projector = InflationProjector::new(0.0);
        assert_eq!(projector.future_nominal_cost_cents(1000, 10), 1000);
        assert_eq!(projector.present_purchasing_power_cents(1000, 10), 1000);
    }

    #[test]
    fn test_zero_years() {
        let projector = InflationProjector::new(5.0);
        assert_eq!(projector.future_nominal_cost_cents(5000, 0), 5000);
        assert_eq!(projector.present_purchasing_power_cents(5000, 0), 5000);
    }

    #[test]
    fn test_future_nominal_cost() {
        let projector = InflationProjector::new(3.0);

        // $10.00 @ 3% for 1 year = $10.30
        assert_eq!(projector.future_nominal_cost_cents(1000, 1), 1030);

        // $10.00 @ 3% for 2 years = $10.00 * 1.03 * 1.03 = 10.609 -> $10.61
        assert_eq!(projector.future_nominal_cost_cents(1000, 2), 1061);

        // $100,000.00 @ 2.5% for 10 years
        // 10,000,000 * (1.025)^10 = 10,000,000 * 1.2800845 = 12,800,845
        let projector_10yr = InflationProjector::new(2.5);
        assert_eq!(
            projector_10yr.future_nominal_cost_cents(10_000_000, 10),
            12_800_845
        );
    }

    #[test]
    fn test_present_purchasing_power() {
        let projector = InflationProjector::new(3.0);

        // $10.30 in 1 year at 3% = $10.00 today
        assert_eq!(projector.present_purchasing_power_cents(1030, 1), 1000);

        // $10.61 in 2 years at 3% = $10.00 today
        assert_eq!(projector.present_purchasing_power_cents(1061, 2), 1000);

        // 12,800,845 in 10 years at 2.5% = 10,000,000 today
        let projector_10yr = InflationProjector::new(2.5);
        assert_eq!(
            projector_10yr.present_purchasing_power_cents(12_800_845, 10),
            10_000_000
        );
    }
}
