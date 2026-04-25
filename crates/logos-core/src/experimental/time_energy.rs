#![cfg(feature = "nova")]

//! Time Energy Equivalency
//!
//! A simulator that answers the question: "How many hours of my life did this cost?"
//! Based on the concept from "Your Money or Your Life", this translates monetary
//! expenses into the real time energy spent earning that money.

use crate::domain::transaction::Transaction;

/// The time cost of an expense.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimeEnergyCost {
    /// Description of the expense or transaction.
    pub description: String,
    /// The monetary cost in cents.
    pub cost_cents: i64,
    /// The equivalent number of hours required to earn this amount.
    pub hours: u32,
    /// The remaining equivalent number of minutes required.
    pub minutes: u8,
}

/// A converter that translates monetary value into time energy based on wage.
#[derive(Debug, Clone)]
pub struct TimeEnergyConverter {
    hourly_wage_cents: u32,
}

impl TimeEnergyConverter {
    /// Creates a new `TimeEnergyConverter` given a specific hourly wage in cents.
    ///
    /// # Arguments
    /// * `hourly_wage_cents` - The user's hourly wage in cents. Must be greater than 0.
    #[must_use]
    pub const fn new(hourly_wage_cents: u32) -> Option<Self> {
        if hourly_wage_cents == 0 {
            return None;
        }
        Some(Self { hourly_wage_cents })
    }

    /// Creates a new `TimeEnergyConverter` from an annual salary, assuming
    /// a standard 2080 hour work year (40 hours/week * 52 weeks).
    ///
    /// # Arguments
    /// * `annual_salary_cents` - The user's annual salary in cents.
    #[must_use]
    pub const fn from_annual_salary(annual_salary_cents: u64) -> Option<Self> {
        if annual_salary_cents == 0 {
            return None;
        }
        // 2080 hours per year
        #[allow(clippy::cast_possible_truncation)]
        let hourly_wage_cents = (annual_salary_cents / 2080) as u32;
        Self::new(hourly_wage_cents)
    }

    /// Converts an amount in cents into its equivalent time energy.
    #[must_use]
    pub fn convert_amount(&self, description: &str, cost_cents: i64) -> TimeEnergyCost {
        let abs_cost = cost_cents.unsigned_abs();

        let total_minutes = (abs_cost * 60) / u64::from(self.hourly_wage_cents);
        #[allow(clippy::cast_possible_truncation)]
        let hours = (total_minutes / 60) as u32;
        let minutes = (total_minutes % 60) as u8;

        TimeEnergyCost {
            description: description.to_string(),
            cost_cents,
            hours,
            minutes,
        }
    }

    /// Analyzes a transaction and returns the time energy cost of its debits (spending).
    /// If there are multiple debits, it aggregates them into a single cost.
    #[must_use]
    pub fn convert_transaction(&self, transaction: &Transaction) -> TimeEnergyCost {
        let total_spending_cents = transaction
            .postings()
            .iter()
            .filter(|p| p.amount() > 0)
            .map(crate::domain::transaction::Posting::amount)
            .sum::<i64>();

        self.convert_amount(transaction.description(), total_spending_cents)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::account::AccountId;
    use crate::domain::transaction::{Posting, TransactionBuilder};

    #[test]
    fn test_time_energy_conversion() {
        // $20/hr = 2000 cents
        let converter = TimeEnergyConverter::new(2000).unwrap();

        // $10 expense should be 0 hours, 30 minutes
        let cost = converter.convert_amount("Lunch", 1000);
        assert_eq!(cost.hours, 0);
        assert_eq!(cost.minutes, 30);

        // $50 expense should be 2 hours, 30 minutes
        let cost = converter.convert_amount("Game", 5000);
        assert_eq!(cost.hours, 2);
        assert_eq!(cost.minutes, 30);
    }

    #[test]
    fn test_from_annual_salary() {
        // $104,000/yr = $50/hr = 5000 cents
        // 104,000 * 100 = 10,400,000 cents
        let converter = TimeEnergyConverter::from_annual_salary(10_400_000).unwrap();
        assert_eq!(converter.hourly_wage_cents, 5000);

        // $100 expense should be exactly 2 hours
        let cost = converter.convert_amount("Dinner", 10000);
        assert_eq!(cost.hours, 2);
        assert_eq!(cost.minutes, 0);
    }

    #[test]
    fn test_zero_wage() {
        assert!(TimeEnergyConverter::new(0).is_none());
        assert!(TimeEnergyConverter::from_annual_salary(0).is_none());
    }

    #[test]
    fn test_transaction_conversion() {
        // $30/hr = 3000 cents
        let converter = TimeEnergyConverter::new(3000).unwrap();

        let tx = TransactionBuilder::new("Groceries")
            .posting(Posting::credit(AccountId::new("assets:checking").unwrap(), 4500).unwrap())
            .posting(Posting::debit(AccountId::new("expenses:food").unwrap(), 4500).unwrap())
            .build()
            .unwrap();

        // $45 expense at $30/hr = 1.5 hours = 1 hour, 30 minutes
        let cost = converter.convert_transaction(&tx);
        assert_eq!(cost.description, "Groceries");
        assert_eq!(cost.cost_cents, 4500);
        assert_eq!(cost.hours, 1);
        assert_eq!(cost.minutes, 30);
    }
}
