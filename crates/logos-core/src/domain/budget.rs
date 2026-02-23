#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BudgetMonth {
    month_key: String,
    start_balance: i64,
    assigned: i64,
    spent: i64,
}

impl BudgetMonth {
    #[must_use]
    pub fn new(month_key: &str, start_balance: i64, assigned: i64, spent: i64) -> Self {
        Self {
            month_key: month_key.to_owned(),
            start_balance,
            assigned,
            spent,
        }
    }

    #[must_use]
    pub fn month_key(&self) -> &str {
        &self.month_key
    }

    #[must_use]
    pub const fn start_balance(&self) -> i64 {
        self.start_balance
    }

    #[must_use]
    pub const fn assigned(&self) -> i64 {
        self.assigned
    }

    #[must_use]
    pub const fn spent(&self) -> i64 {
        self.spent
    }

    #[must_use]
    pub const fn end_balance(&self) -> i64 {
        rollover_end_balance(self.start_balance, self.assigned, self.spent)
    }
}

#[must_use]
pub const fn rollover_end_balance(start: i64, assigned: i64, spent: i64) -> i64 {
    start + assigned - spent
}
