//! Net worth calculation engine.
//!
//! Provides the math to calculate total net worth from assets and liabilities.

/// Projects total net worth by subtracting total liabilities from total assets.
///
/// This is the standard formula for net worth (`Net Worth = Assets - Liabilities`).
///
/// ## Examples
///
/// ```
/// use logos_reporting::project_net_worth;
///
/// // $15,000 in assets and $5,000 in liabilities gives a net worth of $10,000.
/// let net_worth = project_net_worth(15000_00, 5000_00);
/// assert_eq!(net_worth, 10000_00);
///
/// // More liabilities than assets results in a negative net worth.
/// let underwater = project_net_worth(2000_00, 5000_00);
/// assert_eq!(underwater, -3000_00);
/// ```
#[must_use]
pub const fn project_net_worth(assets_cents: i64, liabilities_cents: i64) -> i64 {
    assets_cents.saturating_sub(liabilities_cents)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_saturate_on_underflow() {
        let variance = project_net_worth(i64::MIN, 1);
        assert_eq!(variance, i64::MIN);
    }

    #[test]
    fn should_saturate_on_overflow() {
        let variance = project_net_worth(i64::MAX, -1);
        assert_eq!(variance, i64::MAX);
    }
}
