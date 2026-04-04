//! Account types and their double-entry accounting behavior.
//!
//! This module defines the primary categories of accounts used in `logos`
//! and their fundamental properties, such as their normal balance signs.
//! Understanding these categories is essential for correct transaction balancing
//! and reporting.

use crate::error::DomainError;

/// A strongly-typed identifier for an account in the ledger.
///
/// Wraps an `Arc<str>` to enforce domain boundaries, prevent stringly-typed
/// parameter mix-ups, and provide cheap zero-cost cloning since accounts
/// are frequently passed and duplicated.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AccountId(std::sync::Arc<str>);

impl AccountId {
    /// Creates a new `AccountId`, trimming whitespace.
    ///
    /// ## Examples
    ///
    /// ```
    /// use logos_core::AccountId;
    ///
    /// let id = AccountId::new(" assets:checking ").unwrap();
    /// assert_eq!(id.as_str(), "assets:checking");
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error when `id` is empty after trimming.
    pub fn new(id: &str) -> Result<Self, DomainError> {
        let trimmed = id.trim();
        if trimmed.is_empty() {
            return Err(DomainError::EmptyAccountId);
        }

        Ok(Self(trimmed.into()))
    }

    /// Retrieves the string representation of the account id.
    ///
    /// ## Examples
    ///
    /// ```
    /// use logos_core::AccountId;
    ///
    /// let id = AccountId::new("assets:checking").unwrap();
    /// assert_eq!(id.as_str(), "assets:checking");
    /// ```
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Represents the five fundamental account types in double-entry bookkeeping.
///
/// Every account in the ledger belongs to one of these types. The type
/// determines whether an increase in the account's value is recorded as a
/// debit (positive) or a credit (negative).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccountType {
    /// Resources owned by the entity (e.g., checking accounts, cash).
    /// Increased by debits, decreased by credits.
    Asset,
    /// Obligations owed to others (e.g., credit cards, loans).
    /// Increased by credits, decreased by debits.
    Liability,
    /// The residual interest after liabilities are deducted from assets.
    /// Increased by credits, decreased by debits.
    Equity,
    /// Revenue earned from operations (e.g., salary, interest).
    /// Increased by credits, decreased by debits.
    Income,
    /// Costs incurred during operations (e.g., groceries, rent).
    /// Increased by debits, decreased by credits.
    Expense,
}

impl AccountType {
    /// Returns the "normal balance" sign for the account type.
    ///
    /// In double-entry bookkeeping, the normal balance is the type of entry
    /// (debit or credit) that increases the account's balance. In `logos`:
    ///
    /// * **Debits are positive (+1)**
    /// * **Credits are negative (-1)**
    ///
    /// Therefore, accounts that are normally increased by debits (Assets, Expenses)
    /// have a normal balance sign of `1`. Accounts normally increased by credits
    /// (Liabilities, Equity, Income) have a normal balance sign of `-1`.
    ///
    /// ## Examples
    ///
    /// ```
    /// use logos_core::domain::account::AccountType;
    ///
    /// // Assets are increased by debits (positive values)
    /// assert_eq!(AccountType::Asset.normal_balance_sign(), 1);
    ///
    /// // Liabilities are increased by credits (negative values)
    /// assert_eq!(AccountType::Liability.normal_balance_sign(), -1);
    /// ```
    #[must_use]
    pub const fn normal_balance_sign(self) -> i8 {
        match self {
            Self::Asset | Self::Expense => 1,
            Self::Liability | Self::Equity | Self::Income => -1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::DomainError;

    #[test]
    fn should_return_positive_one_for_debit_normal_accounts() {
        assert_eq!(
            AccountType::Asset.normal_balance_sign(),
            1,
            "Assets should have a normal debit balance"
        );
        assert_eq!(
            AccountType::Expense.normal_balance_sign(),
            1,
            "Expenses should have a normal debit balance"
        );
    }

    #[test]
    fn should_return_negative_one_for_credit_normal_accounts() {
        assert_eq!(
            AccountType::Liability.normal_balance_sign(),
            -1,
            "Liabilities should have a normal credit balance"
        );
        assert_eq!(
            AccountType::Equity.normal_balance_sign(),
            -1,
            "Equity should have a normal credit balance"
        );
        assert_eq!(
            AccountType::Income.normal_balance_sign(),
            -1,
            "Income should have a normal credit balance"
        );
    }

    #[test]
    fn should_trim_account_id() {
        let id = AccountId::new(" assets:checking ").expect("valid account id");
        assert_eq!(id.as_str(), "assets:checking");
    }

    #[test]
    fn should_reject_empty_account_id() {
        assert_eq!(AccountId::new(""), Err(DomainError::EmptyAccountId));
        assert_eq!(AccountId::new("   "), Err(DomainError::EmptyAccountId));
    }
}
