use crate::error::DomainError;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MonthKey(String);

impl MonthKey {
    pub fn new(id: &str) -> Result<Self, DomainError> {
        let trimmed = id.trim();
        if trimmed.is_empty() {
            return Err(DomainError::EmptyMonthKey);
        }

        Ok(Self(trimmed.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
