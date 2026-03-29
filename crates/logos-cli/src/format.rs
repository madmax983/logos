pub fn format_cents(cents: i64) -> String {
    let is_negative = cents < 0;
    let abs_cents = cents.abs();
    let dollars = abs_cents / 100;
    let remaining_cents = abs_cents % 100;

    let dollar_str = dollars.to_string();
    let mut with_commas = String::new();

    let mut count = 0;
    for c in dollar_str.chars().rev() {
        if count != 0 && count % 3 == 0 {
            with_commas.insert(0, ',');
        }
        with_commas.insert(0, c);
        count += 1;
    }

    let sign = if is_negative { "-" } else { "" };
    format!("{}${}.{:02}", sign, with_commas, remaining_cents)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_cents() {
        assert_eq!(format_cents(0), "$0.00");
        assert_eq!(format_cents(100), "$1.00");
        assert_eq!(format_cents(-100), "-$1.00");
        assert_eq!(format_cents(123456), "$1,234.56");
        assert_eq!(format_cents(-123456), "-$1,234.56");
        assert_eq!(format_cents(123456789), "$1,234,567.89");
    }
}
