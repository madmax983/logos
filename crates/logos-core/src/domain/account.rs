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
