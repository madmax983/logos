//! Categorization and envelope organization structures.
//!
//! Budget categories in `logos` follow a hierarchical structure:
//! Every [`Category`] belongs to a [`CategoryGroup`].
//! This structure helps organize expenses logically and is the foundation
//! for the envelope budgeting feature.

/// A normalized identifier for a [`CategoryGroup`].
///
/// It is derived from the group's name by converting to lowercase, trimming
/// whitespace, and replacing spaces with hyphens.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CategoryGroupId(String);

impl CategoryGroupId {
    /// Creates a normalized `CategoryGroupId` from a string.
    ///
    /// ## Examples
    ///
    /// ```
    /// use logos_core::domain::category::CategoryGroupId;
    ///
    /// let id = CategoryGroupId::from_name(" Living Expenses ");
    /// // Internally stores "living-expenses"
    /// ```
    #[must_use]
    pub fn from_name(name: &str) -> Self {
        let normalized = name.trim().to_ascii_lowercase().replace(' ', "-");
        Self(normalized)
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
    /// # Errors
    ///
    /// Returns an error when `name` is empty after trimming.
    pub fn new(name: &str) -> Result<Self, &'static str> {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return Err("category group name cannot be empty");
        }

        Ok(Self {
            id: CategoryGroupId::from_name(trimmed),
            name: trimmed.to_owned(),
        })
    }

    #[must_use]
    pub const fn id(&self) -> &CategoryGroupId {
        &self.id
    }

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
    /// # Errors
    ///
    /// Returns an error when `name` is empty after trimming.
    pub fn new(group_id: CategoryGroupId, name: &str) -> Result<Self, &'static str> {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return Err("category name cannot be empty");
        }

        Ok(Self {
            group_id,
            name: trimmed.to_owned(),
        })
    }

    #[must_use]
    pub const fn group_id(&self) -> &CategoryGroupId {
        &self.group_id
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}
