#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccountType {
    Asset,
    Liability,
    Equity,
    Income,
    Expense,
}

impl AccountType {
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
}
