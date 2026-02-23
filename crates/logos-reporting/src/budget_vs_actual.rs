#[must_use]
pub const fn project_budget_variance(budget_cents: i64, actual_cents: i64) -> i64 {
    budget_cents - actual_cents
}
