//! Opportunity Cost Analyzer
//!
//! A simulator that takes recurring expenses and calculates their long-term
//! opportunity cost if that money had been invested instead. It answers the
//! question: "What is the true cost of this monthly subscription over 10 years?"

use crate::experimental::cashflow_projector::RecurringTemplate;

/// The result of an opportunity cost analysis for a single recurring expense.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpportunityCostResult {
    /// The description of the recurring expense.
    pub description: String,
    /// The monthly cost in cents.
    pub monthly_cost_cents: i64,
    /// The projected future value in cents if invested.
    pub future_value_cents: i64,
}

/// An analyzer that calculates the future value of recurring expenses.
#[derive(Debug, Clone)]
pub struct OpportunityCostAnalyzer {
    annual_return_pct: f64,
    years: u8,
}

impl OpportunityCostAnalyzer {
    /// Creates a new `OpportunityCostAnalyzer`.
    ///
    /// # Arguments
    /// * `annual_return_pct` - Expected real annual return (e.g., 7.0 for 7%).
    /// * `years` - How many years into the future to project the opportunity cost.
    #[must_use]
    pub const fn new(annual_return_pct: f64, years: u8) -> Self {
        Self {
            annual_return_pct,
            years,
        }
    }

    /// Calculates the opportunity cost of a recurring expense template.
    ///
    /// Uses the future value of a series formula to determine how much the
    /// monthly payments would grow to if invested at the expected annual return.
    #[must_use]
    pub fn analyze(&self, template: &RecurringTemplate) -> OpportunityCostResult {
        let r = self.annual_return_pct / 100.0 / 12.0;
        let n = f64::from(self.years) * 12.0;
        #[allow(clippy::cast_precision_loss)]
        let p = template.amount_cents as f64;

        let future_value = if r > 0.0 {
            p * (((1.0 + r).powf(n) - 1.0) / r)
        } else {
            p * n
        };

        #[allow(clippy::cast_possible_truncation)]
        let future_value_cents = future_value.round() as i64;

        OpportunityCostResult {
            description: template.description.clone(),
            monthly_cost_cents: template.amount_cents,
            future_value_cents,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opportunity_cost_calculation() {
        let analyzer = OpportunityCostAnalyzer::new(7.0, 10);

        let template = RecurringTemplate {
            description: "Netflix".to_string(),
            amount_cents: 15_99, // $15.99/mo
            credit_account: "assets:checking".to_string(),
            debit_account: "expenses:entertainment".to_string(),
        };

        let result = analyzer.analyze(&template);

        assert_eq!(result.description, "Netflix");
        assert_eq!(result.monthly_cost_cents, 15_99);

        // $15.99/mo for 10 years at 7% annual return
        // r = 0.07 / 12 = 0.0058333
        // n = 120
        // FV = 15.99 * (((1 + 0.0058333)^120 - 1) / 0.0058333) = ~2768.80
        // Let's assert it is close
        assert!((result.future_value_cents - 276_880).abs() < 1000);
    }

    #[test]
    fn test_opportunity_cost_zero_return() {
        let analyzer = OpportunityCostAnalyzer::new(0.0, 5);

        let template = RecurringTemplate {
            description: "Gym".to_string(),
            amount_cents: 50_00, // $50.00/mo
            credit_account: "assets:checking".to_string(),
            debit_account: "expenses:health".to_string(),
        };

        let result = analyzer.analyze(&template);

        // $50/mo for 5 years at 0% return is exactly 5 * 12 * $50 = $3000
        assert_eq!(result.future_value_cents, 300_000);
    }

    #[test]
    fn test_opportunity_cost_zero_years() {
        let analyzer = OpportunityCostAnalyzer::new(10.0, 0);

        let template = RecurringTemplate {
            description: "Coffee".to_string(),
            amount_cents: 10_000,
            credit_account: "assets:checking".to_string(),
            debit_account: "expenses:food".to_string(),
        };

        let result = analyzer.analyze(&template);

        assert_eq!(result.future_value_cents, 0);
    }
}
