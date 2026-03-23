#![allow(clippy::cast_precision_loss)]

#[must_use]
pub fn format_cents(cents: i64) -> String {
    let dollars = cents.abs() / 100;
    let cents_part = cents.abs() % 100;

    let d_str = dollars.to_string();
    let chars: Vec<char> = d_str.chars().collect();
    let len = chars.len();

    let mut s = String::new();
    for (i, c) in chars.into_iter().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            s.push(',');
        }
        s.push(c);
    }

    let sign = if cents < 0 { "-" } else { "" };
    format!("{sign}${s}.{cents_part:02}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_cents_correctly() {
        assert_eq!(format_cents(0), "$0.00");
        assert_eq!(format_cents(50), "$0.50");
        assert_eq!(format_cents(100), "$1.00");
        assert_eq!(format_cents(1_000), "$10.00");
        assert_eq!(format_cents(100_000), "$1,000.00");
        assert_eq!(format_cents(150_000_000), "$1,500,000.00");
        assert_eq!(format_cents(-150_000_000), "-$1,500,000.00");
    }
}
