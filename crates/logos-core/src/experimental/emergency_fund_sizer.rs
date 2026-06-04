#![cfg(feature = "nova")]

//! Emergency Fund Sizer
//!
//! Calculates the exact size of an emergency fund needed by isolating
//! "essential" expenses from historical transactions, and projects the true
//! runway using the `RunwaySimulator`.
//!
//! 🌟 Nova Mashup: Combines `CategoryTrendAnalyzer` (to find historical burn)
//! with `RunwaySimulator` (to calculate survival months).

use crate::domain::category::CategoryGroupId;
use crate::domain::transaction::Transaction;
use crate::experimental::category_trends::CategoryTrendAnalyzer;
use crate::experimental::runway_simulator::RunwaySimulator;
use std::collections::HashMap;

/// Indicates whether an expense category is essential for survival.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExpenseUrgency {
    /// Must pay (housing, groceries, utilities)
    Essential,
    /// Can be cut during an emergency (entertainment, dining out)
    Discretionary,
}

/// A report detailing the emergency fund analysis.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmergencyFundReport {
    /// Average essential monthly expenses in cents.
    pub essential_monthly_burn_cents: i64,
    /// Average discretionary monthly expenses in cents.
    pub discretionary_monthly_burn_cents: i64,
    /// The runway if keeping the current lifestyle.
    pub current_lifestyle_runway_months: u32,
    /// The runway if cutting all discretionary expenses.
    pub essential_only_runway_months: u32,
    /// The amount needed to reach the target months of essential runway.
    pub target_fund_size_cents: i64,
    /// The deficit (or surplus) to reach the target essential runway.
    pub deficit_to_target_cents: i64,
}

/// Analyzes transactions to determine emergency fund requirements.
#[derive(Debug, Default)]
pub struct EmergencyFundSizer {
    trend_analyzer: CategoryTrendAnalyzer,
    urgency_map: HashMap<CategoryGroupId, ExpenseUrgency>,
    months_of_history: u32,
}

impl EmergencyFundSizer {
    /// Creates a new `EmergencyFundSizer`.
    #[must_use]
    pub fn new(months_of_history: u32) -> Self {
        Self {
            trend_analyzer: CategoryTrendAnalyzer::new(),
            urgency_map: HashMap::new(),
            months_of_history,
        }
    }

    /// Exposes the underlying trend analyzer so accounts can be mapped.
    pub const fn trend_analyzer_mut(&mut self) -> &mut CategoryTrendAnalyzer {
        &mut self.trend_analyzer
    }

    /// Categorizes a group as essential or discretionary.
    pub fn set_urgency(&mut self, group_id: CategoryGroupId, urgency: ExpenseUrgency) {
        self.urgency_map.insert(group_id, urgency);
    }

    /// Analyzes the transactions and calculates the emergency fund report.
    #[must_use]
    #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
    pub fn analyze(
        &self,
        transactions: &[Transaction],
        current_liquid_assets_cents: i64,
        target_months: u32,
        inflation_pct: f64,
    ) -> EmergencyFundReport {
        let trends = self
            .trend_analyzer
            .compute_spending_by_category(transactions);

        let mut total_essential = 0;
        let mut total_discretionary = 0;

        for (group_id, total_spent) in trends {
            let urgency = self
                .urgency_map
                .get(&group_id)
                .unwrap_or(&ExpenseUrgency::Discretionary);
            match urgency {
                ExpenseUrgency::Essential => total_essential += total_spent,
                ExpenseUrgency::Discretionary => total_discretionary += total_spent,
            }
        }

        let months = if self.months_of_history == 0 {
            1
        } else {
            self.months_of_history
        };

        let avg_essential = total_essential / i64::from(months);
        let avg_discretionary = total_discretionary / i64::from(months);
        let avg_total_burn = avg_essential + avg_discretionary;

        let current_runway =
            RunwaySimulator::new(current_liquid_assets_cents, avg_total_burn, inflation_pct)
                .calculate_runway();
        let essential_runway =
            RunwaySimulator::new(current_liquid_assets_cents, avg_essential, inflation_pct)
                .calculate_runway();

        let mut target_size = 0;
        let mut current_burn = avg_essential as f64;
        let monthly_inflation_rate = if inflation_pct > 0.0 {
            (1.0 + inflation_pct / 100.0).powf(1.0 / 12.0) - 1.0
        } else {
            0.0
        };

        for _ in 0..target_months {
            target_size += current_burn.round() as i64;
            current_burn *= 1.0 + monthly_inflation_rate;
        }

        let deficit = target_size - current_liquid_assets_cents;

        EmergencyFundReport {
            essential_monthly_burn_cents: avg_essential,
            discretionary_monthly_burn_cents: avg_discretionary,
            current_lifestyle_runway_months: current_runway.months,
            essential_only_runway_months: essential_runway.months,
            target_fund_size_cents: target_size,
            deficit_to_target_cents: deficit,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::account::AccountId;
    use crate::domain::transaction::{Posting, TransactionBuilder};

    #[test]
    fn test_emergency_fund_analysis() {
        let mut sizer = EmergencyFundSizer::new(2); // 2 months of history

        let rent_acc = AccountId::new("expenses:rent").unwrap();
        let food_acc = AccountId::new("expenses:food").unwrap();
        let movie_acc = AccountId::new("expenses:movies").unwrap();
        let checking_acc = AccountId::new("assets:checking").unwrap();

        let housing_group = CategoryGroupId::from_name("Housing").unwrap();
        let living_group = CategoryGroupId::from_name("Living").unwrap();
        let fun_group = CategoryGroupId::from_name("Entertainment").unwrap();

        sizer
            .trend_analyzer_mut()
            .map_account(rent_acc.clone(), housing_group.clone());
        sizer
            .trend_analyzer_mut()
            .map_account(food_acc.clone(), living_group.clone());
        sizer
            .trend_analyzer_mut()
            .map_account(movie_acc.clone(), fun_group.clone());

        sizer.set_urgency(housing_group, ExpenseUrgency::Essential);
        sizer.set_urgency(living_group, ExpenseUrgency::Essential);
        sizer.set_urgency(fun_group, ExpenseUrgency::Discretionary);

        let tx1 = TransactionBuilder::new("Rent")
            .posting(Posting::debit(rent_acc, 200_000).unwrap())
            .posting(Posting::credit(checking_acc.clone(), 200_000).unwrap())
            .build()
            .unwrap();

        let tx2 = TransactionBuilder::new("Groceries")
            .posting(Posting::debit(food_acc, 100_000).unwrap())
            .posting(Posting::credit(checking_acc.clone(), 100_000).unwrap())
            .build()
            .unwrap();

        let tx3 = TransactionBuilder::new("Movies")
            .posting(Posting::debit(movie_acc, 50_000).unwrap())
            .posting(Posting::credit(checking_acc, 50_000).unwrap())
            .build()
            .unwrap();

        let transactions = vec![tx1, tx2, tx3];

        let report = sizer.analyze(&transactions, 500_000, 6, 0.0);

        assert_eq!(report.essential_monthly_burn_cents, 150_000);
        assert_eq!(report.discretionary_monthly_burn_cents, 25_000);

        assert_eq!(report.current_lifestyle_runway_months, 3);
        assert_eq!(report.essential_only_runway_months, 4);

        assert_eq!(report.target_fund_size_cents, 900_000);
        assert_eq!(report.deficit_to_target_cents, 400_000);
    }
}
