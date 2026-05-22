#[must_use]
pub fn us_timestamp(us: i64) -> String {
    chrono::DateTime::from_timestamp_micros(us).map_or_else(
        || us.to_string(),
        |dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
    )
}

/// Formats currency with commas, e.g., $1,500,000.00
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
    fn test_us_timestamp_out_of_bounds() {
        // Provide extreme values that will map_or_else into the fallback string conversion
        assert_eq!(us_timestamp(i64::MAX), i64::MAX.to_string());
        assert_eq!(us_timestamp(i64::MIN), i64::MIN.to_string());
    }

    #[test]
    fn test_currency_formatting() {
        assert_eq!(currency(150_000_000), "$1,500,000.00");
        assert_eq!(currency(-150_000_000), "-$1,500,000.00");
        assert_eq!(currency(100), "$1.00");
        assert_eq!(currency(-50), "-$0.50");
        assert_eq!(currency(0), "$0.00");
    }
}
