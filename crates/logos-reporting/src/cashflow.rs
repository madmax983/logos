//! Cashflow projection engine.
//!
//! Provides the math to calculate net cash flow from income and expenses.

/// Projects net cash flow by subtracting total expenses from total income.
///
/// A positive cashflow means you are bringing in more money than you are
/// spending (a surplus). A negative cashflow means you are spending more
/// than you are bringing in (a deficit).
///
/// ## Examples
///
/// ```
/// use logos_reporting::project_cashflow;
///
/// // Income of $5,000 and expenses of $4,000 results in a +$1,000 cashflow.
/// let surplus = project_cashflow(5000_00, 4000_00);
/// assert_eq!(surplus, 1000_00);
///
/// // Income of $3,000 and expenses of $3,500 results in a -$500 cashflow.
/// let deficit = project_cashflow(3000_00, 3500_00);
/// assert_eq!(deficit, -500_00);
/// ```
#[must_use]
pub const fn project_cashflow(income_cents: i64, expense_cents: i64) -> i64 {
    income_cents.saturating_sub(expense_cents)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_saturate_on_overflow() {
        let variance = project_cashflow(i64::MIN, 1);
        assert_eq!(variance, i64::MIN);
    }
}
