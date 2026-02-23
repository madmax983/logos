#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RegisterEntry {
    delta_cents: i64,
}

impl RegisterEntry {
    #[must_use]
    pub const fn new(delta_cents: i64) -> Self {
        Self { delta_cents }
    }

    #[must_use]
    pub const fn delta_cents(&self) -> i64 {
        self.delta_cents
    }
}

#[must_use]
pub fn project_register_balance(opening_balance_cents: i64, entries: &[RegisterEntry]) -> i64 {
    entries
        .iter()
        .fold(opening_balance_cents, |balance, entry| {
            balance + entry.delta_cents()
        })
}
