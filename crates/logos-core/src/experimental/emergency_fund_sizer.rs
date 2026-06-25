#![cfg(feature = "nova")]

//! Emergency Fund Sizer
//!
//! Calculates survival runway using historical categorization of "essential" vs "discretionary" spending.

use crate::domain::category::CategoryGroupId;
use crate::domain::transaction::Transaction;
use crate::experimental::category_trends::CategoryTrendAnalyzer;
use crate::experimental::runway_simulator::{RunwayResult, RunwaySimulator};
use std::collections::HashSet;

/// Sizer to calculate realistic emergency fund runway.
#[derive(Debug, Clone)]
pub struct EmergencyFundSizer {
    essential_groups: HashSet<CategoryGroupId>,
}

impl EmergencyFundSizer {
    /// Creates a new Emergency Fund Sizer with the specified essential category groups.
    #[must_use]
    pub fn new(essential_groups: HashSet<CategoryGroupId>) -> Self {
        Self { essential_groups }
    }

    /// Calculates the true runway based on historical essential spending.
    #[must_use]
    pub fn calculate_runway(
        &self,
        analyzer: &CategoryTrendAnalyzer,
        transactions: &[Transaction],
        historical_months: u32,
        liquid_assets_cents: i64,
        annual_inflation_pct: f64,
    ) -> RunwayResult {
        let trends = analyzer.compute_spending_by_category(transactions);

        let total_essential_spent: i64 = trends
            .into_iter()
            .filter(|(group_id, _)| self.essential_groups.contains(group_id))
            .map(|(_, amount)| amount)
            .fold(0, i64::saturating_add);

        let average_monthly_burn = if historical_months > 0 {
            total_essential_spent / i64::from(historical_months)
        } else {
            0
        };

        let simulator = RunwaySimulator::new(
            liquid_assets_cents,
            average_monthly_burn,
            annual_inflation_pct,
        );
        simulator.calculate_runway()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::account::AccountId;
    use crate::domain::transaction::{Posting, TransactionBuilder};

    #[test]
    fn test_emergency_fund_runway() {
        let housing = CategoryGroupId::from_name("Housing").unwrap();
        let entertainment = CategoryGroupId::from_name("Entertainment").unwrap();

        let mut essential = HashSet::new();
        essential.insert(housing.clone());

        let sizer = EmergencyFundSizer::new(essential);

        let mut analyzer = CategoryTrendAnalyzer::new();
        analyzer.map_account(AccountId::new("expenses:rent").unwrap(), housing);
        analyzer.map_account(AccountId::new("expenses:movies").unwrap(), entertainment);

        let tx1 = TransactionBuilder::new("Rent")
            .posting(Posting::debit(AccountId::new("expenses:rent").unwrap(), 200_000).unwrap())
            .posting(Posting::credit(AccountId::new("assets:checking").unwrap(), 200_000).unwrap())
            .build()
            .unwrap();

        let tx2 = TransactionBuilder::new("Movies")
            .posting(Posting::debit(AccountId::new("expenses:movies").unwrap(), 50_000).unwrap())
            .posting(Posting::credit(AccountId::new("assets:checking").unwrap(), 50_000).unwrap())
            .build()
            .unwrap();

        // Total essential spent = 200,000. Over 2 months = 100,000/mo.
        // Liquid assets = 300,000. Runway should be 3 months.
        let result = sizer.calculate_runway(&analyzer, &[tx1, tx2], 2, 300_000, 0.0);

        assert_eq!(result.months, 3);
        assert_eq!(result.total_burned_cents, 300_000);
    }
}
