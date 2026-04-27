//! Envelope budgeting and rollover calculations.
//!
//! The `budget` module provides structures for managing month-scoped envelopes.
//! In envelope budgeting, money from a previous month rolls over into the next.
//! This ensures that unspent funds remain available, and overspending must be
//! explicitly covered.

/// Represents a single envelope's budget state for a specific month.
///
/// Contains the rolled-over start balance, the amount newly assigned to the
/// envelope during the month, and the total amount spent.
///
/// ## Examples
///
/// ```
/// use logos_core::BudgetMonth;
///
/// let month = BudgetMonth::new("2026-03", 100_00, 500_00, 200_00);
/// assert_eq!(month.end_balance(), 400_00);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BudgetMonth {
    month_key: String,
    start_balance: i64,
    assigned: i64,
    spent: i64,
}

impl BudgetMonth {
    /// Creates a new `BudgetMonth` record.
    ///
    /// In envelope budgeting, money from previous months flows forward. `start_balance` represents
    /// the unspent funds (or overspending deficit) from the previous month. `assigned` represents
    /// the new funds injected into this envelope for the current month. `spent` represents what
    /// has left the envelope.
    ///
    /// ## Examples
    ///
    /// ```
    /// use logos_core::BudgetMonth;
    ///
    /// // February's envelope had $100 unspent at the end of the month.
    /// // In March, we "rollover" that $100 as the `start_balance`.
    /// // We assign a new $500 to the envelope for March.
    /// // We spend $200 during March.
    /// let march_budget = BudgetMonth::new("2026-03", 100_00, 500_00, 200_00);
    ///
    /// // At the end of March, $400 remains. This will become April's `start_balance`.
    /// assert_eq!(march_budget.end_balance(), 400_00);
    /// ```
    #[must_use]
    pub fn new(month_key: &str, start_balance: i64, assigned: i64, spent: i64) -> Self {
        Self {
            month_key: month_key.to_owned(),
            start_balance,
            assigned,
            spent,
        }
    }

    /// Retrieves the month key for this budget envelope.
    ///
    /// ## Examples
    ///
    /// ```
    /// use logos_core::BudgetMonth;
    ///
    /// let budget = BudgetMonth::new("2026-03", 100_00, 500_00, 200_00);
    /// assert_eq!(budget.month_key(), "2026-03");
    /// ```
    #[must_use]
    pub fn month_key(&self) -> &str {
        &self.month_key
    }

    /// Retrieves the rolled-over balance from the previous month.
    ///
    /// Useful for determining if there are surplus funds carrying forward,
    /// or if the envelope started in a deficit due to overspending.
    ///
    /// ## Examples
    ///
    /// ```
    /// use logos_core::BudgetMonth;
    ///
    /// let budget = BudgetMonth::new("2026-03", 100_00, 500_00, 200_00);
    /// assert_eq!(budget.start_balance(), 100_00);
    /// ```
    #[must_use]
    pub const fn start_balance(&self) -> i64 {
        self.start_balance
    }

    /// Retrieves the amount of new money injected into this envelope for the current month.
    ///
    /// This represents explicit budgeting decisions made this month, separate
    /// from rolled-over funds.
    ///
    /// ## Examples
    ///
    /// ```
    /// use logos_core::BudgetMonth;
    ///
    /// let budget = BudgetMonth::new("2026-03", 100_00, 500_00, 200_00);
    /// assert_eq!(budget.assigned(), 500_00);
    /// ```
    #[must_use]
    pub const fn assigned(&self) -> i64 {
        self.assigned
    }

    /// Retrieves the total amount of money that has left the envelope this month.
    ///
    /// This is used to track burn rate against the sum of starting and assigned funds.
    ///
    /// ## Examples
    ///
    /// ```
    /// use logos_core::BudgetMonth;
    ///
    /// let budget = BudgetMonth::new("2026-03", 100_00, 500_00, 200_00);
    /// assert_eq!(budget.spent(), 200_00);
    /// ```
    #[must_use]
    pub const fn spent(&self) -> i64 {
        self.spent
    }

    /// Calculates the final balance of the envelope at the end of the month.
    ///
    /// This value will become the `start_balance` for the next month.
    ///
    /// ## Examples
    ///
    /// ```
    /// use logos_core::BudgetMonth;
    ///
    /// let budget = BudgetMonth::new("2026-03", 100_00, 500_00, 200_00);
    /// assert_eq!(budget.end_balance(), 400_00);
    /// ```
    #[must_use]
    pub const fn end_balance(&self) -> i64 {
        rollover_end_balance(self.start_balance, self.assigned, self.spent)
    }
}

/// Calculates the rolling envelope balance for the end of a period.
///
/// Funds available in the envelope equal the starting balance plus any newly
/// assigned funds. Subtracting the spent amount yields the final rollover balance.
/// If the exact result is outside `i64` bounds, the value is clamped to
/// `i64::MIN`/`i64::MAX`.
///
/// ## Examples
///
/// ```
/// use logos_core::rollover_end_balance;
///
/// // Start with $100 (10000 cents), assign $50 (5000 cents), spend $120 (12000 cents).
/// // Remaining balance should be $30 (3000 cents).
/// let end = rollover_end_balance(10000, 5000, 12000);
/// assert_eq!(end, 3000);
///
/// // Overspending results in a negative rollover balance.
/// // Start with $0, assign $0, spend $10.
/// let overspent = rollover_end_balance(0, 0, 1000);
/// assert_eq!(overspent, -1000);
/// ```
#[must_use]
#[allow(clippy::cast_possible_truncation)]
pub const fn rollover_end_balance(start: i64, assigned: i64, spent: i64) -> i64 {
    let end = start as i128 + assigned as i128 - spent as i128;
    if end > i64::MAX as i128 {
        i64::MAX
    } else if end < i64::MIN as i128 {
        i64::MIN
    } else {
        end as i64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn should_return_max_when_rollover_balance_is_exactly_i64_max() {
        assert_eq!(rollover_end_balance(i64::MAX, 0, 0), i64::MAX);
    }

    #[test]
    fn should_return_min_when_rollover_balance_is_exactly_i64_min() {
        assert_eq!(rollover_end_balance(i64::MIN, 0, 0), i64::MIN);
    }

    proptest! {
        #[test]
        #[allow(clippy::cast_possible_truncation)]
        fn test_rollover_end_balance_never_panics_and_clamps(start in any::<i64>(), assigned in any::<i64>(), spent in any::<i64>()) {
            let result = rollover_end_balance(start, assigned, spent);

            let expected = i128::from(start) + i128::from(assigned) - i128::from(spent);
            if expected > i128::from(i64::MAX) {
                assert_eq!(result, i64::MAX);
            } else if expected < i128::from(i64::MIN) {
                assert_eq!(result, i64::MIN);
            } else {
                assert_eq!(result, expected as i64);
            }
        }
    }
}

#[cfg(test)]
mod sentinel_tests {
    use super::*;

    #[test]
    fn should_return_exactly_i64_max_when_rollover_balance_is_exactly_i64_max() {
        assert_eq!(rollover_end_balance(i64::MAX, 0, 0), i64::MAX);
    }

    #[test]
    fn should_clamp_when_rollover_balance_is_exactly_i64_max_plus_one() {
        assert_eq!(rollover_end_balance(i64::MAX, 1, 0), i64::MAX);
    }

    #[test]
    fn should_return_exactly_i64_min_when_rollover_balance_is_exactly_i64_min() {
        assert_eq!(rollover_end_balance(i64::MIN, 0, 0), i64::MIN);
    }

    #[test]
    fn should_clamp_when_rollover_balance_is_exactly_i64_min_minus_one() {
        assert_eq!(rollover_end_balance(i64::MIN, 0, 1), i64::MIN);
    }
}
