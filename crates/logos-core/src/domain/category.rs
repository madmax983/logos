#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CategoryGroupId(String);

impl CategoryGroupId {
    #[must_use]
    pub fn from_name(name: &str) -> Self {
        let normalized = name.trim().to_ascii_lowercase().replace(' ', "-");
        Self(normalized)
    }
}

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
