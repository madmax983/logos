use std::collections::HashMap;
use std::fmt::Write;

use crate::domain::account::AccountId;
use crate::domain::transaction::{Posting, TransactionBuilder};

/// Represents a template for a recurring transaction in cashflow projection.
///
/// This template is used to automatically generate future `Transaction`s.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecurringTemplate {
    /// A description for the recurring template transaction.
    pub description: String,
    /// The absolute amount transferred in cents.
    pub amount_cents: i64,
    /// The account string identifier to credit.
    pub credit_account: String,
    /// The account string identifier to debit.
    pub debit_account: String,
}

/// A projector to simulate future cashflows and account balances.
///
/// It takes an initial set of balances and recurring templates,
/// and projects the state of accounts over a number of periods (e.g., months).
#[derive(Debug, Clone, Default)]
pub struct CashflowProjector {
    initial_balances: HashMap<String, i64>,
    recurring_templates: Vec<RecurringTemplate>,
}

impl CashflowProjector {
    /// Creates a new, empty cashflow projector.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the starting balance for a specific account.
    pub fn set_initial_balance(&mut self, account: &str, balance_cents: i64) {
        self.initial_balances
            .insert(account.to_owned(), balance_cents);
    }

    /// Adds a recurring transaction template to the projection.
    pub fn add_recurring_template(&mut self, template: RecurringTemplate) {
        self.recurring_templates.push(template);
    }

    /// Projects account balances after `periods` number of iterations.
    ///
    /// In each period, all recurring templates generate a transaction.
    /// Returns a map of final account balances.
    #[must_use]
    pub fn project_balances(&self, periods: u16) -> HashMap<String, i64> {
        let mut current_balances = self.initial_balances.clone();

        for _ in 0..periods {
            for template in &self.recurring_templates {
                let Ok(credit_account) = AccountId::new(&template.credit_account) else {
                    continue;
                };
                let Ok(debit_account) = AccountId::new(&template.debit_account) else {
                    continue;
                };
                let Ok(credit_posting) = Posting::credit(credit_account, template.amount_cents)
                else {
                    continue;
                };
                let Ok(debit_posting) = Posting::debit(debit_account, template.amount_cents) else {
                    continue;
                };

                let tx_res = TransactionBuilder::new(&template.description)
                    .posting(credit_posting)
                    .posting(debit_posting)
                    .build();

                if let Ok(tx) = tx_res {
                    for posting in tx.postings() {
                        *current_balances
                            .entry(posting.account().as_str().to_owned())
                            .or_insert(0) += posting.amount();
                    }
                }
            }
        }

        current_balances
    }

    /// Generates a report showing the projected balances.
    #[must_use]
    pub fn generate_report(&self, periods: u16) -> String {
        let balances = self.project_balances(periods);
        let mut output = String::new();
        let _ = writeln!(
            &mut output,
            "📊 Cashflow Projection Report ({periods} periods)"
        );

        let mut sorted_accounts: Vec<_> = balances.keys().collect();
        sorted_accounts.sort();

        for account in sorted_accounts {
            let cents = balances[account];
            #[allow(clippy::cast_precision_loss)]
            let dollars = cents as f64 / 100.0;
            let _ = writeln!(&mut output, "{account}: ${dollars:.2}");
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cashflow_projection_balances() {
        let mut projector = CashflowProjector::new();
        projector.set_initial_balance("assets:checking", 100_000); // $1000

        projector.add_recurring_template(RecurringTemplate {
            description: "Salary".to_string(),
            amount_cents: 50_000, // $500
            credit_account: "income:salary".to_string(),
            debit_account: "assets:checking".to_string(),
        });

        projector.add_recurring_template(RecurringTemplate {
            description: "Rent".to_string(),
            amount_cents: 30_000, // $300
            credit_account: "assets:checking".to_string(),
            debit_account: "expenses:rent".to_string(),
        });

        // After 2 periods:
        // Checking: 1000 + (500*2) - (300*2) = 1400 ($1400.00)
        // Income: -500 * 2 = -1000 (-$1000.00)
        // Rent: 300 * 2 = 600 ($600.00)
        let balances = projector.project_balances(2);

        assert_eq!(*balances.get("assets:checking").unwrap_or(&0), 140_000);
        assert_eq!(*balances.get("income:salary").unwrap_or(&0), -100_000);
        assert_eq!(*balances.get("expenses:rent").unwrap_or(&0), 60_000);
    }

    #[test]
    fn test_cashflow_projection_report() {
        let mut projector = CashflowProjector::new();
        projector.set_initial_balance("assets:checking", 100_000);

        projector.add_recurring_template(RecurringTemplate {
            description: "Salary".to_string(),
            amount_cents: 50_000,
            credit_account: "income:salary".to_string(),
            debit_account: "assets:checking".to_string(),
        });

        let report = projector.generate_report(1);
        assert!(report.contains("assets:checking: $1500.00"));
        assert!(report.contains("income:salary: $-500.00"));
    }
}
