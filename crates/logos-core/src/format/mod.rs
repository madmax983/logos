//! String formatting utilities for display.
//!
//! # The Human Interface
//!
//! While the engine strictly uses integer cents and microseconds for correctness,
//! humans expect to see dollars, commas, and readable dates. This module bridges
//! the gap between machine precision and human understanding.

/// Converts a microsecond UNIX timestamp into a human-readable UTC date string.
///
/// We use microseconds (`us`) as the standard time unit across the system to maintain
/// precision when sorting high-frequency transaction logs. This function renders that
/// timestamp into a standard `YYYY-MM-DD HH:MM:SS UTC` format for display purposes.
/// If the timestamp is invalid and cannot be mapped to a datetime, it gracefully falls
/// back to returning the raw microsecond integer as a string.
///
/// ## Examples
///
/// ```
/// use logos_core::format::us_timestamp;
///
/// let ts = 1715424000000000; // May 11, 2024 10:40:00 UTC
/// assert_eq!(us_timestamp(ts), "2024-05-11 10:40:00 UTC");
/// ```
#[must_use]
pub fn us_timestamp(us: i64) -> String {
    chrono::DateTime::from_timestamp_micros(us).map_or_else(
        || us.to_string(),
        |dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
    )
}

/// Formats currency with commas, e.g., $1,500,000.00
///
/// It strictly formats integer cents into a dollar representation. This ensures that
/// we never encounter floating-point rounding errors when displaying financial data.
///
/// ## Examples
///
/// ```
/// use logos_core::format::currency;
///
/// assert_eq!(currency(1500000), "$15,000.00");
/// assert_eq!(currency(-50), "-$0.50");
/// ```
///
/// ⚡ Bolt Optimization:
/// Previously, this function used `String::insert(0, c)` repeatedly within a loop,
/// causing O(n^2) shifts of all existing bytes per character. By pre-allocating the string capacity,
/// iterating forward over the bytes, and using `push`, we achieve O(n) performance
/// and eliminate intermediate allocations on the formatting hot path.
#[must_use]
pub fn currency(cents: i64) -> String {
    let sign = if cents < 0 { "-" } else { "" };
    let abs_cents = cents.abs();
    let dollars = abs_cents / 100;
    let remainder = abs_cents % 100;

    let dollars_str = dollars.to_string();
    let bytes = dollars_str.as_bytes();
    let len = bytes.len();

    let mut out = String::with_capacity(len + len / 3 + 6);
    out.push_str(sign);
    out.push('$');

    for (i, &b) in bytes.iter().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            out.push(',');
        }
        out.push(b as char);
    }

    let _ = std::fmt::write(&mut out, format_args!(".{remainder:02}"));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_currency_formatting() {
        assert_eq!(currency(150_000_000), "$1,500,000.00");
        assert_eq!(currency(-150_000_000), "-$1,500,000.00");
        assert_eq!(currency(100), "$1.00");
        assert_eq!(currency(-50), "-$0.50");
        assert_eq!(currency(0), "$0.00");
    }
}
