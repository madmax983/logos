#[must_use]
pub const fn project_cashflow(income_cents: i64, expense_cents: i64) -> i64 {
    income_cents - expense_cents
}
