use crate::experimental::inflation::InflationProjector;
use crate::experimental::monte_carlo::MonteCarloProjector;

/// Represents the analyzed opportunity cost of a purchase.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpportunityCostReport {
    /// The nominal future cost of the purchase if it had been invested instead (median outcome).
    pub future_value_median_cents: i64,
    /// The present purchasing power equivalent of the future value (median outcome).
    pub present_purchasing_power_median_cents: i64,
}

/// Analyzes the opportunity cost of an expense.
#[derive(Debug, Clone)]
pub struct OpportunityAnalyzer {
    annual_return: f64,
    annual_volatility: f64,
    annual_inflation_pct: f64,
    seed: u64,
}

impl OpportunityAnalyzer {
    /// Creates a new `OpportunityAnalyzer`.
    #[must_use]
    pub const fn new(
        annual_return: f64,
        annual_volatility: f64,
        annual_inflation_pct: f64,
        seed: u64,
    ) -> Self {
        Self {
            annual_return,
            annual_volatility,
            annual_inflation_pct,
            seed,
        }
    }

    /// Analyzes the opportunity cost of spending `expense_cents` instead of investing it for `years` years.
    #[must_use]
    pub fn analyze_purchase(
        &self,
        expense_cents: i64,
        years: u16,
        paths: u32,
    ) -> OpportunityCostReport {
        if expense_cents <= 0 || years == 0 {
            return OpportunityCostReport {
                future_value_median_cents: expense_cents,
                present_purchasing_power_median_cents: expense_cents,
            };
        }

        // Project the growth of the expense if invested instead
        // The MonteCarloProjector runs on a monthly basis
        let months = years * 12;
        let monte_carlo = MonteCarloProjector::new(
            expense_cents,
            0, // No ongoing contribution, just a lump sum
            self.annual_return,
            self.annual_volatility,
            self.seed,
        );
        let mc_result = monte_carlo.run(months, paths);

        // Calculate the present purchasing power of the median future value
        let inflation_projector = InflationProjector::new(self.annual_inflation_pct);
        let present_value_median =
            inflation_projector.present_purchasing_power_cents(mc_result.median_cents, years);

        OpportunityCostReport {
            future_value_median_cents: mc_result.median_cents,
            present_purchasing_power_median_cents: present_value_median,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_expense() {
        let analyzer = OpportunityAnalyzer::new(0.07, 0.15, 3.0, 42);
        let result = analyzer.analyze_purchase(0, 10, 100);

        assert_eq!(result.future_value_median_cents, 0);
        assert_eq!(result.present_purchasing_power_median_cents, 0);
    }

    #[test]
    fn test_zero_years() {
        let analyzer = OpportunityAnalyzer::new(0.07, 0.15, 3.0, 42);
        let result = analyzer.analyze_purchase(10_000, 0, 100);

        assert_eq!(result.future_value_median_cents, 10_000);
        assert_eq!(result.present_purchasing_power_median_cents, 10_000);
    }

    #[test]
    fn test_opportunity_cost() {
        // $10,000 expense, 10 years
        // 7% return, 15% volatility, 3% inflation
        let analyzer = OpportunityAnalyzer::new(0.07, 0.15, 3.0, 42);
        let result = analyzer.analyze_purchase(1_000_000, 10, 1000);

        // $10,000 at 7% for 10 years should be around $19,671
        // $19,671 in 10 years adjusted for 3% inflation should be around $14,637
        // So the median should be more than the initial expense but less than the future value
        assert!(result.future_value_median_cents > 1_000_000);
        assert!(result.present_purchasing_power_median_cents > 1_000_000);
        assert!(result.present_purchasing_power_median_cents < result.future_value_median_cents);
    }

    #[test]
    fn test_zero_paths() {
        let analyzer = OpportunityAnalyzer::new(0.07, 0.15, 3.0, 42);
        let result = analyzer.analyze_purchase(10_000, 10, 0);

        // When paths is 0, Monte Carlo returns the initial cents as median
        // Then we calculate present value of 10_000 in 10 years at 3% inflation
        let inflation = InflationProjector::new(3.0);
        let expected_pv = inflation.present_purchasing_power_cents(10_000, 10);

        assert_eq!(result.future_value_median_cents, 10_000);
        assert_eq!(result.present_purchasing_power_median_cents, expected_pv);
    }
}
