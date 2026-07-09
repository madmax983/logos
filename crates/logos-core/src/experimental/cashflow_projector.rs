//! Cashflow Projection Module
//!
//! # Seeing Around Corners
//!
//! While standard accounting tells you where your money *went*, the [`CashflowProjector`]
//! helps you see where it is *going*. By feeding it your current account balances and
//! a list of [`RecurringTemplate`] definitions (like rent, salary, and subscriptions),
//! it simulates your ledger forward in time. This prevents surprises, ensuring you don't
//! accidentally drain your checking account before the next paycheck arrives.
//!
//! **Note:** This module is experimental and primarily used for rough estimation rather than
//! strict, penny-perfect forecasting.

use std::collections::HashMap;
use std::fmt::Write;

use crate::domain::account::AccountId;
use crate::domain::transaction::{Posting, TransactionBuilder};

/// Represents a template for a recurring transaction in cashflow projection.
///
/// This template acts as a blueprint to automatically generate future [`crate::domain::transaction::Transaction`]s
/// during the simulation. It maps directly to expected recurring events like rent or paychecks.
///
/// ## Examples
///
/// ```
/// use logos_core::RecurringTemplate;
///
/// let template = RecurringTemplate {
///     description: "Netflix".to_string(),
///     amount_cents: 15_99, // $15.99
///     credit_account: "assets:checking".to_string(),
///     debit_account: "expenses:entertainment".to_string(),
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecurringTemplate {
    /// A description for the recurring template, effectively the future transaction's payee or note.
    pub description: String,
    /// The amount of the recurring flow, in cents.
    pub amount_cents: i64,
    /// The account to credit (source of funds).
    pub credit_account: String,
    /// The account to debit (destination of funds).
    pub debit_account: String,
}

/// A projector to simulate future cashflows and account balances.
///
/// It takes an initial set of balances and a list of [`RecurringTemplate`]s,
/// and projects the state of accounts over a number of periods (e.g., months).
///
/// ## Examples
///
/// ```
/// use logos_core::{CashflowProjector, RecurringTemplate};
///
/// let mut projector = CashflowProjector::new();
/// projector.set_initial_balance("assets:checking", 100_000); // Start with $1000
///
/// projector.add_recurring_template(RecurringTemplate {
///     description: "Salary".to_string(),
///     amount_cents: 200_000,
///     credit_account: "income:salary".to_string(),
///     debit_account: "assets:checking".to_string(),
/// });
///
/// // After 3 months, we should have an extra $6000 ($2000 * 3) in checking.
/// let balances = projector.project_balances(3);
/// assert_eq!(*balances.get("assets:checking").unwrap(), 700_000); // 1000 + 6000 = $7000
/// ```
#[derive(Debug, Clone, Default)]
pub struct CashflowProjector {
    initial_balances: HashMap<String, i64>,
    recurring_templates: Vec<RecurringTemplate>,
}

impl CashflowProjector {
    /// Creates a fresh, empty projector ready to be populated with balances and templates.
    ///
    /// ## Examples
    ///
    /// ```
    /// use logos_core::CashflowProjector;
    ///
    /// let projector = CashflowProjector::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Seeds the simulation with the current reality of your ledger.
    ///
    /// The projector needs to know your starting balances in cents so it can correctly
    /// add or subtract future cashflows.
    ///
    /// ## Examples
    ///
    /// ```
    /// use logos_core::CashflowProjector;
    ///
    /// let mut projector = CashflowProjector::new();
    /// projector.set_initial_balance("assets:checking", 500_000); // $5000.00
    /// ```
    pub fn set_initial_balance(&mut self, account: &str, balance_cents: i64) {
        self.initial_balances
            .insert(account.to_owned(), balance_cents);
    }

    /// Adds a recurring transaction template (like a monthly bill or paycheck) to the simulation.
    ///
    /// ## Examples
    ///
    /// ```
    /// use logos_core::{CashflowProjector, RecurringTemplate};
    ///
    /// let mut projector = CashflowProjector::new();
    /// projector.add_recurring_template(RecurringTemplate {
    ///     description: "Internet Bill".to_string(),
    ///     amount_cents: 80_00,
    ///     credit_account: "assets:checking".to_string(),
    ///     debit_account: "expenses:utilities".to_string(),
    /// });
    /// ```
    pub fn add_recurring_template(&mut self, template: RecurringTemplate) {
        self.recurring_templates.push(template);
    }

    /// Projects account balances after `periods` number of iterations.
    ///
    /// In each period, the projector processes every [`RecurringTemplate`] and simulates
    /// a valid double-entry transaction, updating the running balances of the involved accounts.
    ///
    /// ## Examples
    ///
    /// ```
    /// use logos_core::{CashflowProjector, RecurringTemplate};
    ///
    /// let mut projector = CashflowProjector::new();
    /// projector.set_initial_balance("assets:savings", 10_000_00); // $10k
    /// projector.add_recurring_template(RecurringTemplate {
    ///     description: "Save!".to_string(),
    ///     amount_cents: 500_00, // $500
    ///     credit_account: "assets:checking".to_string(),
    ///     debit_account: "assets:savings".to_string(),
    /// });
    ///
    /// let balances = projector.project_balances(6); // Project 6 months out
    /// // $10,000 + ($500 * 6) = $13,000
    /// assert_eq!(*balances.get("assets:savings").unwrap(), 13_000_00);
    /// ```
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
                        // ⚡ Bolt Optimization: Avoid unconditional `.to_owned()` string allocation.
                        // We first check if the key exists using a slice, and only allocate
                        // a new owned String if we need to insert a new entry.
                        if let Some(balance) = current_balances.get_mut(posting.account().as_str())
                        {
                            *balance += posting.amount();
                        } else {
                            current_balances
                                .insert(posting.account().as_str().to_owned(), posting.amount());
                        }
                    }
                }
            }
        }

        current_balances
    }

    /// Generates a human-readable text report showing the projected balances for all affected accounts.
    ///
    /// Converts internal cent values to dollars for readability.
    ///
    /// ## Examples
    ///
    /// ```
    /// use logos_core::{CashflowProjector, RecurringTemplate};
    ///
    /// let mut projector = CashflowProjector::new();
    /// projector.set_initial_balance("assets:checking", 100_000);
    ///
    /// let report = projector.generate_report(1);
    /// assert!(report.contains("assets:checking: $1000.00"));
    /// ```
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
