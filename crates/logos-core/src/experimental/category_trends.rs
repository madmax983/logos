use std::collections::HashMap;

use crate::domain::account::AccountId;
use crate::domain::category::CategoryGroupId;
use crate::domain::transaction::Transaction;

/// Analyzes transactions to determine spending trends across category groups.
///
/// This provides a foundational layer for tracking expenses grouped by high-level
/// budget categories over time.
#[derive(Debug, Default)]
pub struct CategoryTrendAnalyzer {
    account_to_category: HashMap<AccountId, CategoryGroupId>,
}

impl CategoryTrendAnalyzer {
    /// Creates a new analyzer with an empty category mapping.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Maps a specific account to a category group.
    ///
    /// When computing trends, debits to this account will be aggregated
    /// under the associated category group.
    pub fn map_account(&mut self, account: AccountId, category_group: CategoryGroupId) {
        self.account_to_category.insert(account, category_group);
    }

    /// Analyzes a set of transactions and returns the total spent per category group.
    ///
    /// Only debits (positive amounts) to mapped accounts are included in the totals.
    /// Credits are ignored in this simple expenditure analyzer.
    #[must_use]
    pub fn compute_spending_by_category(
        &self,
        transactions: &[Transaction],
    ) -> HashMap<CategoryGroupId, i64> {
        let mut trends: HashMap<CategoryGroupId, i64> = HashMap::new();

        for tx in transactions {
            for posting in tx.postings() {
                // We only care about debits (spending)
                if posting.amount() > 0 {
                    if let Some(group_id) = self.account_to_category.get(posting.account()) {
                        *trends.entry(group_id.clone()).or_insert(0) += posting.amount();
                    }
                }
            }
        }

        trends
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::transaction::{Posting, TransactionBuilder};

    #[test]
    fn test_spending_by_category() {
        let mut analyzer = CategoryTrendAnalyzer::new();

        let rent_acc = AccountId::new("expenses:rent").unwrap();
        let food_acc = AccountId::new("expenses:food").unwrap();
        let checking_acc = AccountId::new("assets:checking").unwrap();
        let salary_acc = AccountId::new("income:salary").unwrap();

        let housing_group = CategoryGroupId::from_name("Housing").unwrap();
        let living_group = CategoryGroupId::from_name("Living").unwrap();

        analyzer.map_account(rent_acc.clone(), housing_group.clone());
        analyzer.map_account(food_acc.clone(), living_group.clone());

        // Tx 1: Rent Payment
        let tx1 = TransactionBuilder::new("Rent")
            .posting(Posting::debit(rent_acc, 200_000).unwrap())
            .posting(Posting::credit(checking_acc.clone(), 200_000).unwrap())
            .build()
            .unwrap();

        // Tx 2: Groceries
        let tx2 = TransactionBuilder::new("Groceries")
            .posting(Posting::debit(food_acc.clone(), 15_000).unwrap())
            .posting(Posting::credit(checking_acc.clone(), 15_000).unwrap())
            .build()
            .unwrap();

        // Tx 3: Eating Out
        let tx3 = TransactionBuilder::new("Restaurant")
            .posting(Posting::debit(food_acc, 8_000).unwrap())
            .posting(Posting::credit(checking_acc.clone(), 8_000).unwrap())
            .build()
            .unwrap();

        // Tx 4: Salary Income (Should be ignored by analyzer since it's a credit to income, and checking debit isn't mapped to an expense category)
        let tx4 = TransactionBuilder::new("Salary")
            .posting(Posting::debit(checking_acc, 500_000).unwrap())
            .posting(Posting::credit(salary_acc, 500_000).unwrap())
            .build()
            .unwrap();

        let transactions = vec![tx1, tx2, tx3, tx4];

        let trends = analyzer.compute_spending_by_category(&transactions);

        assert_eq!(trends.len(), 2);
        assert_eq!(trends.get(&housing_group).copied(), Some(200_000));
        assert_eq!(trends.get(&living_group).copied(), Some(23_000));
    }
}
