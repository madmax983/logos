use core::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainError {
    EmptyAccountId,
    EmptyTransactionId,
    EmptyCategoryGroupName,
    EmptyCategoryName,
    EmptyTransactionDescription,
    EmptyTransactionPostings,
    EmptyCorrectionReason,
    CorrectionCannotSupersedeSelf,
    InvalidAllocationTotal { total: u16 },
    InvalidHaircutPercentage { tier: &'static str, percentage: u8 },
    InvalidHaircutOrdering { short: u8, medium: u8, long: u8 },
    InvalidDebitAmount { amount: i64 },
    InvalidCreditAmount { amount: i64 },
    UnbalancedTransaction { total: i64 },
    AmountOverflow,
}

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyAccountId => {
                write!(f, "account id cannot be empty")
            }
            Self::EmptyTransactionId => {
                write!(f, "transaction id cannot be empty")
            }
            Self::EmptyCategoryGroupName => {
                write!(f, "category group name cannot be empty")
            }
            Self::EmptyCategoryName => {
                write!(f, "category name cannot be empty")
            }
            Self::EmptyTransactionDescription => {
                write!(f, "transaction description cannot be empty")
            }
            Self::EmptyTransactionPostings => {
                write!(f, "transaction must contain at least one posting")
            }
            Self::EmptyCorrectionReason => {
                write!(f, "correction reason cannot be empty")
            }
            Self::CorrectionCannotSupersedeSelf => {
                write!(f, "correction cannot supersede itself")
            }
            Self::InvalidAllocationTotal { total } => {
                write!(f, "allocation percentages must sum to 100, got {total}")
            }
            Self::InvalidHaircutPercentage { tier, percentage } => {
                write!(
                    f,
                    "haircut percentage for {tier} tier must be in 0..=100, got {percentage}"
                )
            }
            Self::InvalidHaircutOrdering {
                short,
                medium,
                long,
            } => {
                write!(
                    f,
                    "haircut tiers must be non-decreasing by horizon (short <= medium <= long), got {short}, {medium}, {long}"
                )
            }
            Self::InvalidDebitAmount { amount } => {
                write!(f, "debit amount must be greater than zero, got {amount}")
            }
            Self::InvalidCreditAmount { amount } => {
                write!(f, "credit amount must be greater than zero, got {amount}")
            }
            Self::UnbalancedTransaction { total } => {
                write!(
                    f,
                    "transaction must be balanced to zero, but total was {total}"
                )
            }
            Self::AmountOverflow => {
                write!(f, "transaction amount calculation resulted in an overflow")
            }
        }
    }
}

impl std::error::Error for DomainError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_display_empty_transaction_description() {
        assert_eq!(
            DomainError::EmptyTransactionDescription.to_string(),
            "transaction description cannot be empty"
        );
    }

    #[test]
    fn should_display_empty_transaction_postings() {
        assert_eq!(
            DomainError::EmptyTransactionPostings.to_string(),
            "transaction must contain at least one posting"
        );
    }

    #[test]
    fn should_display_empty_account_id() {
        assert_eq!(
            DomainError::EmptyAccountId.to_string(),
            "account id cannot be empty"
        );
    }

    #[test]
    fn should_display_empty_correction_reason() {
        assert_eq!(
            DomainError::EmptyCorrectionReason.to_string(),
            "correction reason cannot be empty"
        );
    }

    #[test]
    fn should_display_empty_transaction_id() {
        assert_eq!(
            DomainError::EmptyTransactionId.to_string(),
            "transaction id cannot be empty"
        );
    }

    #[test]
    fn should_display_correction_cannot_supersede_self() {
        assert_eq!(
            DomainError::CorrectionCannotSupersedeSelf.to_string(),
            "correction cannot supersede itself"
        );
    }

    #[test]
    fn should_display_invalid_allocation_total() {
        assert_eq!(
            DomainError::InvalidAllocationTotal { total: 105 }.to_string(),
            "allocation percentages must sum to 100, got 105"
        );
    }

    #[test]
    fn should_display_unbalanced_transaction() {
        assert_eq!(
            DomainError::UnbalancedTransaction { total: -500 }.to_string(),
            "transaction must be balanced to zero, but total was -500"
        );
    }

    #[test]
    fn should_display_invalid_haircut_percentage() {
        assert_eq!(
            DomainError::InvalidHaircutPercentage {
                tier: "long",
                percentage: 120
            }
            .to_string(),
            "haircut percentage for long tier must be in 0..=100, got 120"
        );
    }

    #[test]
    fn should_display_invalid_haircut_ordering() {
        assert_eq!(
            DomainError::InvalidHaircutOrdering {
                short: 50,
                medium: 40,
                long: 60
            }
            .to_string(),
            "haircut tiers must be non-decreasing by horizon (short <= medium <= long), got 50, 40, 60"
        );
    }

    #[test]
    fn should_display_invalid_debit_amount() {
        assert_eq!(
            DomainError::InvalidDebitAmount { amount: 0 }.to_string(),
            "debit amount must be greater than zero, got 0"
        );
    }

    #[test]
    fn should_display_invalid_credit_amount() {
        assert_eq!(
            DomainError::InvalidCreditAmount { amount: -1 }.to_string(),
            "credit amount must be greater than zero, got -1"
        );
    }
}
