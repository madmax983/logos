use crate::domain::category::CategoryGroupId;
use crate::domain::transaction::Transaction;
use crate::experimental::category_trends::CategoryTrendAnalyzer;
use crate::experimental::runway_simulator::RunwaySimulator;

/// Represents the runway gained by eliminating a specific category group's expenses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunwayGained {
    /// The category group that was eliminated.
    pub category_group: CategoryGroupId,
    /// The amount of runway gained in months.
    pub months_gained: u32,
    /// The original monthly burn for this category.
    pub category_burn_cents: i64,
}

/// Analyzes how eliminating specific expense categories affects overall financial runway.
#[derive(Debug)]
pub struct RunwayExtenderAnalyzer {
    liquid_assets_cents: i64,
    total_monthly_burn_cents: i64,
    annual_inflation_pct: f64,
    category_trends: CategoryTrendAnalyzer,
}

impl RunwayExtenderAnalyzer {
    /// Creates a new `RunwayExtenderAnalyzer`.
    #[must_use]
    pub const fn new(
        liquid_assets_cents: i64,
        total_monthly_burn_cents: i64,
        annual_inflation_pct: f64,
        category_trends: CategoryTrendAnalyzer,
    ) -> Self {
        Self {
            liquid_assets_cents,
            total_monthly_burn_cents,
            annual_inflation_pct,
            category_trends,
        }
    }

    /// Calculates the runway gained by eliminating each category found in the provided transactions.
    #[must_use]
    pub fn analyze(&self, transactions: &[Transaction]) -> Vec<RunwayGained> {
        let spending_by_category = self
            .category_trends
            .compute_spending_by_category(transactions);

        let base_simulator = RunwaySimulator::new(
            self.liquid_assets_cents,
            self.total_monthly_burn_cents,
            self.annual_inflation_pct,
        );
        let base_runway = base_simulator.calculate_runway().months;

        let mut results = Vec::new();

        for (group_id, amount) in spending_by_category {
            let new_burn = self.total_monthly_burn_cents.saturating_sub(amount);

            let sim = RunwaySimulator::new(
                self.liquid_assets_cents,
                new_burn,
                self.annual_inflation_pct,
            );
            let new_runway = sim.calculate_runway().months;

            results.push(RunwayGained {
                category_group: group_id,
                months_gained: new_runway.saturating_sub(base_runway),
                category_burn_cents: amount,
            });
        }

        results.sort_by_key(|r| std::cmp::Reverse(r.months_gained));
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::account::AccountId;
    use crate::domain::transaction::{Posting, TransactionBuilder};

    #[test]
    fn test_runway_extender() {
        let mut trends = CategoryTrendAnalyzer::new();
        let entertainment_acc = AccountId::new("expenses:entertainment").unwrap();
        let dining_acc = AccountId::new("expenses:dining").unwrap();
        let checking_acc = AccountId::new("assets:checking").unwrap();

        let entertainment_group = CategoryGroupId::from_name("Entertainment").unwrap();
        let dining_group = CategoryGroupId::from_name("Dining").unwrap();

        trends.map_account(entertainment_acc.clone(), entertainment_group.clone());
        trends.map_account(dining_acc.clone(), dining_group.clone());

        let analyzer = RunwayExtenderAnalyzer::new(
            1_200_000, // $12,000 liquid assets
            200_000,   // $2,000 total monthly burn
            0.0,       // 0% inflation
            trends,
        );

        let tx1 = TransactionBuilder::new("Netflix")
            .posting(Posting::debit(entertainment_acc, 100_000).unwrap())
            .posting(Posting::credit(checking_acc.clone(), 100_000).unwrap())
            .build()
            .unwrap();

        let tx2 = TransactionBuilder::new("Restaurant")
            .posting(Posting::debit(dining_acc, 50_000).unwrap())
            .posting(Posting::credit(checking_acc, 50_000).unwrap())
            .build()
            .unwrap();

        let results = analyzer.analyze(&[tx1, tx2]);

        assert_eq!(results.len(), 2);

        // Base runway: 12k / 2k = 6 months
        // Eliminate Entertainment ($1k): New burn = 1k. Runway: 12k / 1k = 12 months. Gained: 6 months.
        assert_eq!(results[0].category_group, entertainment_group);
        assert_eq!(results[0].months_gained, 6);
        assert_eq!(results[0].category_burn_cents, 100_000);

        // Eliminate Dining ($500): New burn = 1.5k. Runway: 12k / 1.5k = 8 months. Gained: 2 months.
        assert_eq!(results[1].category_group, dining_group);
        assert_eq!(results[1].months_gained, 2);
        assert_eq!(results[1].category_burn_cents, 50_000);
    }
}
