//! Budget variance calculations.
//!
//! # The Reality Check
//!
//! A budget is just a hypothesis until the month ends. This module provides the logic
//! to compare your planned targets against cold, hard reality. It answers the question:
//! *"Did I spend what I said I was going to spend?"*
//!
//! By calculating the variance between the budget limit and the actual spending, it
//! allows the system to determine if an envelope has a surplus (which can be rolled over)
//! or a deficit (which must be covered).

/// Projects the variance between a planned budget and actual activity.
///
/// Positive variance means you spent less than budgeted (under budget).
/// Negative variance means you spent more than budgeted (over budget).
///
/// ## Examples
///
/// ```
/// use logos_reporting::project_budget_variance;
///
/// // Budgeted $1000, actual was $800.
/// // We are under budget by $200 (positive variance).
/// let variance = project_budget_variance(1000_00, 800_00);
/// assert_eq!(variance, 200_00);
///
/// // Budgeted $500, actual was $600.
/// // We are over budget by $100 (negative variance).
/// let variance = project_budget_variance(500_00, 600_00);
/// assert_eq!(variance, -100_00);
/// ```
#[must_use]
pub const fn project_budget_variance(budget_cents: i64, actual_cents: i64) -> i64 {
    budget_cents.saturating_sub(actual_cents)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_saturate_on_overflow() {
        let variance = project_budget_variance(i64::MIN, 1);
        assert_eq!(variance, i64::MIN);
    }
}
