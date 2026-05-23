//! Subscription Snowball Accelerator
//!
//! A mashup of `RecurrenceDetector` and `DebtOptimizer`.
//! It finds recurring subscriptions and calculates how much faster you could
//! pay off your debts (and how much interest you'd save) if you canceled
//! those subscriptions and applied the cash directly to your debt snowball.

use crate::domain::transaction::Transaction;
use crate::experimental::debt_optimizer::{Debt, DebtOptimizer, PayoffResult, PayoffStrategy};
use crate::experimental::recurrence_detector::RecurrenceDetector;

/// The result of accelerating debt payoff with a canceled subscription.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubscriptionSnowballItem {
    pub description: String,
    pub monthly_cost_cents: i64,
    pub accelerated_months: u32,
    pub months_saved: u32,
    pub accelerated_interest_paid_cents: i64,
    pub interest_saved_cents: i64,
}

/// A report detailing baseline debt payoff vs accelerated payoff per subscription.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubscriptionSnowballReport {
    pub baseline: PayoffResult,
    pub items: Vec<SubscriptionSnowballItem>,
}

/// Analyzer that calculates debt payoff acceleration.
#[derive(Debug, Clone)]
pub struct SubscriptionSnowballAccelerator {
    recurrence_detector: RecurrenceDetector,
    base_monthly_payment_cents: i64,
    debts: Vec<Debt>,
}

impl SubscriptionSnowballAccelerator {
    #[must_use]
    pub const fn new(min_occurrences: usize, base_monthly_payment_cents: i64, debts: Vec<Debt>) -> Self {
        Self {
            recurrence_detector: RecurrenceDetector::new(min_occurrences),
            base_monthly_payment_cents,
            debts,
        }
    }

    #[must_use]
    pub fn analyze(
        &self,
        transactions: &[Transaction],
        strategy: PayoffStrategy,
    ) -> SubscriptionSnowballReport {
        let mut base_optimizer = DebtOptimizer::new(self.base_monthly_payment_cents);
        for debt in &self.debts {
            base_optimizer.add_debt(debt.clone());
        }
        let baseline = base_optimizer.simulate(strategy);

        let recurring_templates = self.recurrence_detector.detect(transactions);
        let mut items = Vec::new();

        for template in recurring_templates {
            let mut accel_optimizer =
                DebtOptimizer::new(self.base_monthly_payment_cents + template.amount_cents);
            for debt in &self.debts {
                accel_optimizer.add_debt(debt.clone());
            }
            let accel_result = accel_optimizer.simulate(strategy);

            items.push(SubscriptionSnowballItem {
                description: template.description.clone(),
                monthly_cost_cents: template.amount_cents,
                accelerated_months: accel_result.total_months,
                months_saved: baseline
                    .total_months
                    .saturating_sub(accel_result.total_months),
                accelerated_interest_paid_cents: accel_result.total_interest_paid_cents,
                interest_saved_cents: baseline
                    .total_interest_paid_cents
                    .saturating_sub(accel_result.total_interest_paid_cents),
            });
        }

        SubscriptionSnowballReport { baseline, items }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::account::AccountId;
    use crate::domain::transaction::{Posting, TransactionBuilder};
    use crate::experimental::debt_optimizer::{Debt, PayoffStrategy};

    #[test]
    fn test_subscription_snowball() {
        let mut transactions = Vec::new();

        // 3 occurrences of Streaming Service ($15.00/mo)
        for _ in 0..3 {
            let tx = TransactionBuilder::new("Streaming")
                .posting(Posting::credit(AccountId::new("assets:checking").unwrap(), 1500).unwrap())
                .posting(
                    Posting::debit(AccountId::new("expenses:entertainment").unwrap(), 1500)
                        .unwrap(),
                )
                .build()
                .unwrap();
            transactions.push(tx);
        }

        let debts = vec![Debt {
            name: "Credit Card".to_string(),
            balance_cents: 200_000, // $2,000
            interest_rate_pct: 20,
            min_payment_cents: 5_000, // $50
        }];

        // Base budget is $100/mo. Canceling streaming frees up $15/mo.
        let accelerator = SubscriptionSnowballAccelerator::new(3, 10_000, debts);
        let report = accelerator.analyze(&transactions, PayoffStrategy::Snowball);

        // Base: $2000 at 20% paying $100/mo takes 25 months, paying $453.05 interest.
        // Accel: $2000 at 20% paying $115/mo takes 21 months, paying $381.47 interest.
        assert_eq!(report.baseline.total_months, 25);
        assert_eq!(report.items.len(), 1);
        assert_eq!(report.items[0].description, "Streaming");
        assert_eq!(report.items[0].months_saved, 4);
        assert_eq!(report.items[0].interest_saved_cents, 7158); // 45305 - 38147 = 7158
    }
}
