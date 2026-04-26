//! Budget categories and grouping logic.
//!
//! # The Organization of Envelopes
//!
//! In the `logos` envelope budgeting system, expenses are not just tracked; they
//! are proactively organized. Every [`Category`] represents a specific budget envelope
//! (e.g., "Rent") and belongs to a broader [`CategoryGroup`] (e.g., "Housing").
//!
//! This structure helps organize expenses logically and is the foundation
//! for calculating budget rollups and variances.

use crate::error::DomainError;
use std::sync::Arc;

/// A normalized identifier for a [`CategoryGroup`].
///
/// It is derived from the group's name by converting to lowercase, trimming
/// whitespace, and replacing internal whitespace runs with hyphens.
///
/// Uses `Arc<str>` internally to provide zero-cost cloning (reducing heap
/// allocations) across budgeting logic where IDs are frequently duplicated.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CategoryGroupId(Arc<str>);

impl CategoryGroupId {
    /// Creates a normalized `CategoryGroupId` from a string.
    ///
    /// Internal whitespace is collapsed to a single `-`.
    ///
    /// # Errors
    ///
    /// Returns an error when `name` is empty after trimming and normalization.
    ///
    /// ## Examples
    ///
    /// ```
    /// use logos_core::domain::category::CategoryGroupId;
    ///
    /// let id = CategoryGroupId::from_name(" Living Expenses ")?;
    /// assert_eq!(id.as_str(), "living-expenses");
    /// # Ok::<(), logos_core::DomainError>(())
    /// ```
    pub fn from_name(name: &str) -> Result<Self, DomainError> {
        let mut normalized = String::new();

        for segment in name.split_whitespace() {
            if !normalized.is_empty() {
                normalized.push('-');
            }
            normalized.push_str(&segment.to_ascii_lowercase());
        }

        if normalized.is_empty() {
            return Err(DomainError::EmptyCategoryGroupName);
        }

        Ok(Self(normalized.into()))
    }

    /// Retrieves the string representation of the category group id.
    ///
    /// ## Examples
    ///
    /// ```
    /// use logos_core::domain::category::CategoryGroupId;
    ///
    /// let id = CategoryGroupId::from_name("Housing").unwrap();
    /// assert_eq!(id.as_str(), "housing");
    /// ```
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A high-level bucket used to organize related budget categories.
///
/// For example, a `CategoryGroup` named "Housing" might contain categories
/// like "Rent" and "Utilities".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CategoryGroup {
    id: CategoryGroupId,
    name: String,
}

impl CategoryGroup {
    /// Creates a budget category group with a normalized identifier derived from `name`.
    ///
    /// ## Examples
    ///
    /// ```
    /// use logos_core::domain::category::CategoryGroup;
    ///
    /// let group = CategoryGroup::new(" Housing ").expect("valid name");
    /// assert_eq!(group.name(), "Housing");
    /// assert_eq!(group.id().as_str(), "housing");
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error when `name` is empty after trimming.
    pub fn new(name: &str) -> Result<Self, DomainError> {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return Err(DomainError::EmptyCategoryGroupName);
        }

        Ok(Self {
            id: CategoryGroupId::from_name(trimmed)?,
            name: trimmed.to_owned(),
        })
    }

    /// Exposes the normalized grouping identifier.
    ///
    /// This identifier links related [`Category`] instances together so that
    /// analytical roll-ups and budget reporting can correctly group expenses.
    ///
    /// ## Examples
    ///
    /// ```
    /// use logos_core::domain::category::CategoryGroup;
    ///
    /// let group = CategoryGroup::new("Housing").unwrap();
    /// assert_eq!(group.id().as_str(), "housing");
    /// ```
    #[must_use]
    pub const fn id(&self) -> &CategoryGroupId {
        &self.id
    }

    /// Retrieves the original name of the category.
    ///
    /// ## Examples
    ///
    /// ```
    /// use logos_core::domain::category::{Category, CategoryGroup};
    ///
    /// let group = CategoryGroup::new("Housing").unwrap();
    /// let category = Category::new(group.id().clone(), "Rent").unwrap();
    /// assert_eq!(category.name(), "Rent");
    /// ```
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// A specific budget envelope where funds are assigned and spent.
///
/// A `Category` is always scoped to a parent [`CategoryGroup`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Category {
    group_id: CategoryGroupId,
    name: String,
}

impl Category {
    /// Creates a budget category scoped to the provided category group id.
    ///
    /// ## Examples
    ///
    /// ```
    /// use logos_core::domain::category::{Category, CategoryGroup};
    ///
    /// let group = CategoryGroup::new("Housing").expect("valid group");
    /// let category = Category::new(group.id().clone(), " Rent ").expect("valid category");
    ///
    /// assert_eq!(category.name(), "Rent");
    /// assert_eq!(category.group_id().as_str(), "housing");
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error when `name` is empty after trimming.
    pub fn new(group_id: CategoryGroupId, name: &str) -> Result<Self, DomainError> {
        if group_id.as_str().is_empty() {
            return Err(DomainError::EmptyCategoryGroupName);
        }

        let trimmed = name.trim();
        if trimmed.is_empty() {
            return Err(DomainError::EmptyCategoryName);
        }

        Ok(Self {
            group_id,
            name: trimmed.to_owned(),
        })
    }

    /// Identifies the high-level grouping this envelope belongs to.
    ///
    /// Used during transaction aggregation to roll up specific line-item expenses
    /// into their broader parent budgets (e.g. mapping a "Rent" category back to the "Housing" group).
    ///
    /// ## Examples
    ///
    /// ```
    /// use logos_core::domain::category::{Category, CategoryGroup};
    ///
    /// let group = CategoryGroup::new("Housing").unwrap();
    /// let category = Category::new(group.id().clone(), "Rent").unwrap();
    /// assert_eq!(category.group_id().as_str(), "housing");
    /// ```
    #[must_use]
    pub const fn group_id(&self) -> &CategoryGroupId {
        &self.group_id
    }

    /// Retrieves the original name of the category.
    ///
    /// ## Examples
    ///
    /// ```
    /// use logos_core::domain::category::{Category, CategoryGroup};
    ///
    /// let group = CategoryGroup::new("Housing").unwrap();
    /// let category = Category::new(group.id().clone(), "Rent").unwrap();
    /// assert_eq!(category.name(), "Rent");
    /// ```
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_normalize_category_group_id_from_name() {
        let cases = vec![
            ("Needs", "needs"),
            (" Wants ", "wants"),
            ("True Expenses", "true-expenses"),
            ("  Debt Payments  ", "debt-payments"),
            ("Recurring   Bills", "recurring-bills"),
            ("Food\tDining", "food-dining"),
            ("My-Custom-Category", "my-custom-category"),
        ];

        for (input, expected) in cases {
            let id = CategoryGroupId::from_name(input).expect("valid category group id");
            assert_eq!(
                id.as_str(),
                expected,
                "Expected '{input}' to normalize to '{expected}'"
            );
        }
    }

    #[test]
    fn should_return_error_when_category_group_id_name_is_empty() {
        assert_eq!(
            CategoryGroupId::from_name(""),
            Err(DomainError::EmptyCategoryGroupName)
        );
        assert_eq!(
            CategoryGroupId::from_name("   \t  "),
            Err(DomainError::EmptyCategoryGroupName)
        );
    }

    #[test]
    fn should_create_category_group_successfully() -> Result<(), DomainError> {
        let group = CategoryGroup::new(" True Expenses ")?;
        assert_eq!(group.name(), "True Expenses");
        assert_eq!(group.id().as_str(), "true-expenses");
        Ok(())
    }

    #[test]
    fn should_return_error_when_category_group_name_is_empty() {
        assert_eq!(
            CategoryGroup::new(""),
            Err(DomainError::EmptyCategoryGroupName)
        );
        assert_eq!(
            CategoryGroup::new("   "),
            Err(DomainError::EmptyCategoryGroupName)
        );
    }

    #[test]
    fn should_create_category_successfully() -> Result<(), DomainError> {
        let group_id = CategoryGroupId::from_name("Needs")?;
        let category = Category::new(group_id.clone(), " Rent ")?;
        assert_eq!(category.name(), "Rent");
        assert_eq!(category.group_id(), &group_id);
        Ok(())
    }

    #[test]
    fn should_return_error_when_category_group_id_is_empty() {
        let group_id = CategoryGroupId(String::new().into());
        assert_eq!(
            Category::new(group_id, "Rent"),
            Err(DomainError::EmptyCategoryGroupName)
        );
    }

    #[test]
    fn should_return_error_when_category_name_is_empty() {
        let group_id = CategoryGroupId::from_name("Needs").expect("valid category group id");
        assert_eq!(
            Category::new(group_id.clone(), ""),
            Err(DomainError::EmptyCategoryName)
        );
        assert_eq!(
            Category::new(group_id, "   "),
            Err(DomainError::EmptyCategoryName)
        );
    }

    #[test]
    fn should_return_category_name() {
        let group_id = CategoryGroupId::from_name("Needs").expect("valid category group id");
        let category = Category::new(group_id, "Rent").expect("valid category");
        assert_eq!(category.name(), "Rent");
    }

    #[test]
    fn should_return_category_group_id() {
        let group_id = CategoryGroupId::from_name("Needs").expect("valid category group id");
        let category = Category::new(group_id.clone(), "Rent").expect("valid category");
        assert_eq!(category.group_id(), &group_id);
    }
}
