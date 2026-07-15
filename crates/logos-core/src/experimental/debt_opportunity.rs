#![cfg(feature = "nova")]

//! Debt Opportunity Cost Analyzer
//!
//! A simulator that calculates the opportunity cost of paying off debt vs investing
//! the extra money over a fixed horizon.

use crate::experimental::debt_optimizer::{Debt, DebtOptimizer, PayoffStrategy};

/// The result of a debt opportunity cost analysis.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebtOpportunityResult {
    /// The name of the debt.
    pub debt_name: String,
    /// Months to pay off by only making the minimum payment.
    pub original_months_to_payoff: u32,
    /// Months to pay off when making the extra monthly payment.
    pub accelerated_months_to_payoff: u32,
    /// Total interest saved by paying the debt off early (cents).
    pub interest_saved_cents: i64,
    /// The future value of investing the total payment (minimum + extra)
    /// from the moment the debt is paid off until the end of the investment horizon.
    pub future_value_of_saved_payments_cents: i64,
}

/// An analyzer that combines debt optimization and opportunity cost.
#[derive(Debug, Clone)]
pub struct DebtOpportunityAnalyzer {
    annual_investment_return_pct: f64,
    investment_horizon_years: u8,
}

impl DebtOpportunityAnalyzer {
    /// Creates a new `DebtOpportunityAnalyzer`.
    #[must_use]
    pub const fn new(annual_investment_return_pct: f64, investment_horizon_years: u8) -> Self {
        Self {
            annual_investment_return_pct,
            investment_horizon_years,
        }
    }

    /// Analyzes the opportunity cost of accelerating debt payoff.
    #[must_use]
    pub fn analyze(&self, debt: &Debt, extra_monthly_payment_cents: i64) -> DebtOpportunityResult {
        // 1. Calculate baseline payoff (no extra payments)
        let mut baseline_opt = DebtOptimizer::new(debt.min_payment_cents);
        baseline_opt.add_debt(debt.clone());
        let baseline_result = baseline_opt.simulate(PayoffStrategy::Snowball);

        // 2. Calculate accelerated payoff
        let mut accelerated_opt = DebtOptimizer::new(
            debt.min_payment_cents
                .saturating_add(extra_monthly_payment_cents),
        );
        accelerated_opt.add_debt(debt.clone());
        let accelerated_result = accelerated_opt.simulate(PayoffStrategy::Snowball);

        // 3. Calculate interest saved
        let interest_saved_cents = baseline_result
            .total_interest_paid_cents
            .saturating_sub(accelerated_result.total_interest_paid_cents);

        // 4. Calculate opportunity cost (future value) of investing the freed-up cash flow.
        // We invest (min + extra) for the remaining time in the horizon *after* the accelerated payoff is done.
        let total_horizon_months = u32::from(self.investment_horizon_years) * 12;
        let investing_months = total_horizon_months.saturating_sub(accelerated_result.total_months);

        let final_future_value = if investing_months > 0 {
            let monthly_investment = debt
                .min_payment_cents
                .saturating_add(extra_monthly_payment_cents);
            let r = self.annual_investment_return_pct / 100.0 / 12.0;
            let n = f64::from(investing_months);
            #[allow(clippy::cast_precision_loss)]
            let p = monthly_investment as f64;

            if r > 0.0 {
                p * (((1.0 + r).powf(n) - 1.0) / r)
            } else {
                p * n
            }
        } else {
            0.0
        };

        #[allow(clippy::cast_possible_truncation)]
        DebtOpportunityResult {
            debt_name: debt.name.clone(),
            original_months_to_payoff: baseline_result.total_months,
            accelerated_months_to_payoff: accelerated_result.total_months,
            interest_saved_cents,
            future_value_of_saved_payments_cents: final_future_value.round() as i64,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debt_opportunity_analyzer() {
        // 7% annual return, 10 year horizon
        let analyzer = DebtOpportunityAnalyzer::new(7.0, 10);

        let debt = Debt {
            name: "Car Loan".to_string(),
            balance_cents: 2_000_000, // $20,000
            interest_rate_pct: 5,
            min_payment_cents: 40_000, // $400
        };

        // Analyze putting an extra $100/mo towards the debt
        let result = analyzer.analyze(&debt, 10_000);

        assert_eq!(result.debt_name, "Car Loan");

        // Baseline: $20,000 at 5% paying $400/mo takes ~57 months
        assert!(result.original_months_to_payoff > 50);

        // Accelerated: $20,000 at 5% paying $500/mo takes ~44 months
        assert!(result.accelerated_months_to_payoff < result.original_months_to_payoff);

        // They save interest
        assert!(result.interest_saved_cents > 0);

        // 10 years = 120 months.
        // 120 - 44 = 76 months of investing $500/mo at 7%.
        // FV = 500 * (((1 + 0.07/12)^76 - 1) / (0.07/12)) = ~$47,698
        assert!(result.future_value_of_saved_payments_cents > 4_000_000);
    }

    #[test]
    fn test_debt_opportunity_analyzer_no_investing_time() {
        // Horizon is too short (1 year). The debt takes longer to pay off.
        let analyzer = DebtOpportunityAnalyzer::new(7.0, 1);

        let debt = Debt {
            name: "Big Loan".to_string(),
            balance_cents: 5_000_000,
            interest_rate_pct: 5,
            min_payment_cents: 50_000,
        };

        let result = analyzer.analyze(&debt, 10_000);

        // Future value should be 0 because horizon is over before debt is paid.
        assert_eq!(result.future_value_of_saved_payments_cents, 0);
    }
}
