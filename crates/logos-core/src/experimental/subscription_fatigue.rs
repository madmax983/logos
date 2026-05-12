//! Subscription Fatigue Analyzer
//!
//! Calculates the long-term opportunity cost of recurring subscriptions to help identify "subscription fatigue" and potential areas for savings.
use crate::domain::transaction::Transaction;
use crate::experimental::opportunity_cost::{OpportunityCostAnalyzer, OpportunityCostResult};
use crate::experimental::recurrence_detector::RecurrenceDetector;

/// A summary report detailing the fatigue of recurring subscriptions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubscriptionFatigueReport {
    /// List of identified subscriptions with their individual opportunity costs.
    pub items: Vec<OpportunityCostResult>,
    /// Total cost per month of all subscriptions combined.
    pub total_monthly_cost_cents: i64,
    /// Total future value lost due to all subscriptions combined.
    pub total_opportunity_cost_cents: i64,
}

/// Analyzer that calculates the long-term pain of recurring expenses.
///
/// Combines `RecurrenceDetector` to automatically identify subscriptions and
/// `OpportunityCostAnalyzer` to project how much they are really costing you.
#[derive(Debug, Clone)]
pub struct SubscriptionFatigueAnalyzer {
    recurrence_detector: RecurrenceDetector,
    opportunity_cost_analyzer: OpportunityCostAnalyzer,
}

impl SubscriptionFatigueAnalyzer {
    /// Creates a new `SubscriptionFatigueAnalyzer`.
    ///
    /// # Arguments
    /// * `min_occurrences` - How many times a transaction must appear to be considered a subscription.
    /// * `annual_return_pct` - Expected real annual return (e.g., 7.0 for 7%) if the money was invested instead.
    /// * `years` - How many years into the future to project the opportunity cost.
    #[must_use]
    pub const fn new(min_occurrences: usize, annual_return_pct: f64, years: u8) -> Self {
        Self {
            recurrence_detector: RecurrenceDetector::new(min_occurrences),
            opportunity_cost_analyzer: OpportunityCostAnalyzer::new(annual_return_pct, years),
        }
    }

    /// Analyzes a list of historical transactions to generate a subscription fatigue report.
    #[must_use]
    pub fn analyze(&self, transactions: &[Transaction]) -> SubscriptionFatigueReport {
        let recurring_templates = self.recurrence_detector.detect(transactions);

        let mut items = Vec::new();
        let mut total_monthly = 0;
        let mut total_opportunity = 0;

        for template in recurring_templates {
            let cost_result = self.opportunity_cost_analyzer.analyze(&template);
            total_monthly += cost_result.monthly_cost_cents;
            total_opportunity += cost_result.future_value_cents;
            items.push(cost_result);
        }

        SubscriptionFatigueReport {
            items,
            total_monthly_cost_cents: total_monthly,
            total_opportunity_cost_cents: total_opportunity,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::account::AccountId;
    use crate::domain::transaction::{Posting, TransactionBuilder};

    #[test]
    fn test_subscription_fatigue_analysis() {
        let analyzer = SubscriptionFatigueAnalyzer::new(3, 7.0, 10);

        let mut transactions = Vec::new();

        // 3 occurrences of Netflix ($15.99)
        for _ in 0..3 {
            let tx = TransactionBuilder::new("Netflix")
                .posting(Posting::credit(AccountId::new("assets:checking").unwrap(), 1599).unwrap())
                .posting(
                    Posting::debit(AccountId::new("expenses:entertainment").unwrap(), 1599)
                        .unwrap(),
                )
                .build()
                .unwrap();
            transactions.push(tx);
        }

        // 3 occurrences of Gym ($50.00)
        for _ in 0..3 {
            let tx = TransactionBuilder::new("Gym")
                .posting(Posting::credit(AccountId::new("assets:checking").unwrap(), 5000).unwrap())
                .posting(Posting::debit(AccountId::new("expenses:health").unwrap(), 5000).unwrap())
                .build()
                .unwrap();
            transactions.push(tx);
        }

        // 1 occurrence of Coffee ($5.00) - Should be ignored due to min_occurrences
        let tx = TransactionBuilder::new("Coffee")
            .posting(Posting::credit(AccountId::new("assets:checking").unwrap(), 500).unwrap())
            .posting(Posting::debit(AccountId::new("expenses:food").unwrap(), 500).unwrap())
            .build()
            .unwrap();
        transactions.push(tx);

        let report = analyzer.analyze(&transactions);

        assert_eq!(report.items.len(), 2);

        let gym = report
            .items
            .iter()
            .find(|i| i.description == "Gym")
            .unwrap();
        assert_eq!(gym.monthly_cost_cents, 5000);

        let netflix = report
            .items
            .iter()
            .find(|i| i.description == "Netflix")
            .unwrap();
        assert_eq!(netflix.monthly_cost_cents, 1599);

        assert_eq!(report.total_monthly_cost_cents, 6599);

        // Ensure total opportunity cost is the sum of the individual items
        assert_eq!(
            report.total_opportunity_cost_cents,
            gym.future_value_cents + netflix.future_value_cents
        );
    }
}
