//! Register balance projection.
//!
//! Provides structures and functions to calculate the running balance of an account
//! by applying a sequence of historical delta entries to an opening balance.

/// A single delta (change in value) to apply to a register balance.
///
/// This represents a single transaction or event that alters an account's balance
/// by a specific amount.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RegisterEntry {
    delta_cents: i64,
}

impl RegisterEntry {
    /// Creates a new `RegisterEntry` with the given change in cents.
    ///
    /// ## Examples
    ///
    /// ```
    /// use logos_reporting::RegisterEntry;
    ///
    /// // Represents a deposit of $50.00
    /// let entry = RegisterEntry::new(5000);
    /// assert_eq!(entry.delta_cents(), 5000);
    /// ```
    #[must_use]
    pub const fn new(delta_cents: i64) -> Self {
        Self { delta_cents }
    }

    /// Retrieves the specific change in value (in cents) for this entry.
    ///
    /// Positive values represent inflows (deposits), while negative values
    /// represent outflows (withdrawals).
    ///
    /// ## Examples
    ///
    /// ```
    /// use logos_reporting::RegisterEntry;
    ///
    /// let deposit = RegisterEntry::new(5000);
    /// assert_eq!(deposit.delta_cents(), 5000);
    ///
    /// let withdrawal = RegisterEntry::new(-2000);
    /// assert_eq!(withdrawal.delta_cents(), -2000);
    /// ```
    #[must_use]
    pub const fn delta_cents(&self) -> i64 {
        self.delta_cents
    }
}

/// Projects a final register balance by applying a slice of entries to an opening balance.
///
/// ## Examples
///
/// ```
/// use logos_reporting::{project_register_balance, RegisterEntry};
///
/// let entries = vec![
///     RegisterEntry::new(100_00), // Deposit $100
///     RegisterEntry::new(-50_00), // Withdraw $50
/// ];
///
/// // Start with $500, apply the $100 deposit and $50 withdrawal to get $550.
/// let final_balance = project_register_balance(500_00, &entries);
/// assert_eq!(final_balance, 550_00);
/// ```
#[must_use]
pub fn project_register_balance(opening_balance_cents: i64, entries: &[RegisterEntry]) -> i64 {
    project_register_balance_iter(opening_balance_cents, entries.iter().copied())
}

/// Projects a final register balance by applying an iterator of entries to an opening balance.
///
/// Useful for lazy evaluation or when entries are generated dynamically without allocating a `Vec`.
///
/// ## Examples
///
/// ```
/// use logos_reporting::{project_register_balance_iter, RegisterEntry};
///
/// let entries = [RegisterEntry::new(100_00), RegisterEntry::new(200_00)];
///
/// // Start with $0, add $100 and $200.
/// let final_balance = project_register_balance_iter(0, entries);
/// assert_eq!(final_balance, 300_00);
/// ```
#[must_use]
pub fn project_register_balance_iter<I>(opening_balance_cents: i64, entries: I) -> i64
where
    I: IntoIterator<Item = RegisterEntry>,
{
    entries
        .into_iter()
        .fold(opening_balance_cents, |balance, entry| {
            balance.saturating_add(entry.delta_cents())
        })
}
