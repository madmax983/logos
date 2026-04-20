#[must_use]
pub fn us_timestamp(us: i64) -> String {
    chrono::DateTime::from_timestamp_micros(us).map_or_else(
        || us.to_string(),
        |dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
    )
}

#[must_use]
pub fn currency(cents: i64) -> String {
    let sign = if cents < 0 { "-" } else { "" };
    let abs_cents = cents.abs();
    let dollars = abs_cents / 100;
    let remainder = abs_cents % 100;

    let dollars_str = dollars.to_string();
    let mut with_commas = String::new();
    for (i, c) in dollars_str.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            with_commas.insert(0, ',');
        }
        with_commas.insert(0, c);
    }

    format!("{sign}${with_commas}.{remainder:02}")
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
