#![cfg(feature = "nova")]

//! Purchase Impact Simulator
//!
//! Evaluates the true cost of a large one-time purchase by calculating
//! how many months of retirement it costs you. It answers the question:
//! "Is this $50,000 truck worth working an extra 2 years for?"

/// The result of a purchase impact analysis.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PurchaseImpactResult {
    /// The original purchase amount in cents.
    pub purchase_amount_cents: i64,
    /// The number of months of financial independence delayed by this purchase.
    pub months_delayed: u16,
    /// The opportunity cost future value of the purchase at the target retirement date.
    pub opportunity_cost_cents: i64,
}

/// A simulator that calculates the time-cost of a one-time purchase.
#[derive(Debug, Clone)]
pub struct PurchaseImpactSimulator {
    annual_return_pct: f64,
    monthly_savings_cents: i64,
    years_to_target: u16,
}

impl PurchaseImpactSimulator {
    /// Creates a new `PurchaseImpactSimulator`.
    ///
    /// # Arguments
    /// * `annual_return_pct` - Expected real annual return (e.g., 7.0 for 7%).
    /// * `monthly_savings_cents` - How much is saved each month.
    /// * `years_to_target` - Estimated years until target financial independence.
    #[must_use]
    pub const fn new(
        annual_return_pct: f64,
        monthly_savings_cents: i64,
        years_to_target: u16,
    ) -> Self {
        Self {
            annual_return_pct,
            monthly_savings_cents,
            years_to_target,
        }
    }

    /// Calculates the impact of a one-time purchase.
    ///
    /// Uses compound interest to find the future value of the purchase amount,
    /// then determines how many months of current savings are required to reach that future value.
    #[must_use]
    pub fn evaluate(&self, purchase_amount_cents: i64) -> PurchaseImpactResult {
        if self.monthly_savings_cents <= 0 || purchase_amount_cents <= 0 {
            return PurchaseImpactResult {
                purchase_amount_cents,
                months_delayed: 0,
                opportunity_cost_cents: purchase_amount_cents,
            };
        }

        let r = self.annual_return_pct / 100.0 / 12.0;
        let n = f64::from(self.years_to_target) * 12.0;
        #[allow(clippy::cast_precision_loss)]
        let p = purchase_amount_cents as f64;

        // Future value of the one-time purchase if invested
        let future_value = if r > 0.0 {
            p * (1.0 + r).powf(n)
        } else {
            p
        };

        #[allow(clippy::cast_precision_loss)]
        let m_savings = self.monthly_savings_cents as f64;

        // How many months to save up to the future_value using monthly_savings_cents?
        // Using Future Value of a Series: FV = PMT * (((1 + r)^n - 1) / r)
        // Solve for n: n = ln(FV * r / PMT + 1) / ln(1 + r)
        #[allow(clippy::imprecise_flops, clippy::suboptimal_flops)]
        let months_delayed_f64 = if r > 0.0 {
            ((future_value * r / m_savings) + 1.0).ln() / (1.0 + r).ln()
        } else {
            future_value / m_savings
        };

        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let months_delayed = months_delayed_f64.round() as u16;

        #[allow(clippy::cast_possible_truncation)]
        let opportunity_cost_cents = future_value.round() as i64;

        PurchaseImpactResult {
            purchase_amount_cents,
            months_delayed,
            opportunity_cost_cents,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_purchase_impact_calculation() {
        // 7% return, $1000/mo savings, 10 years to FI
        let simulator = PurchaseImpactSimulator::new(7.0, 100_000, 10);

        // $50,000 truck
        let result = simulator.evaluate(5_000_000);

        assert_eq!(result.purchase_amount_cents, 5_000_000);

        // $50k at 7% for 10 years: FV = 50k * (1 + 0.07/12)^120 = ~100k
        assert!(result.opportunity_cost_cents > 90_000_000);

        // To save $100k at $1k/mo with 7% growth takes ~78 months
        assert!(result.months_delayed > 60 && result.months_delayed < 90);
    }

    #[test]
    fn test_zero_return() {
        let simulator = PurchaseImpactSimulator::new(0.0, 100_000, 5);

        // $10,000 purchase
        let result = simulator.evaluate(1_000_000);

        assert_eq!(result.opportunity_cost_cents, 1_000_000);
        // $10,000 / $1,000 = 10 months
        assert_eq!(result.months_delayed, 10);
    }

    #[test]
    fn test_zero_savings_or_purchase() {
        let simulator = PurchaseImpactSimulator::new(5.0, 0, 10);
        let result = simulator.evaluate(5_000_000);
        assert_eq!(result.months_delayed, 0);

        let simulator2 = PurchaseImpactSimulator::new(5.0, 100_000, 10);
        let result2 = simulator2.evaluate(0);
        assert_eq!(result2.months_delayed, 0);
    }
}