#[must_use]
pub fn us_timestamp(us: i64) -> String {
    chrono::DateTime::from_timestamp_micros(us).map_or_else(
        || us.to_string(),
        |dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
    )
}

#[must_use]
#[allow(clippy::cast_precision_loss)]
pub fn currency(cents: i64) -> String {
    if cents < 0 {
        format!("-${:.2}", (-cents as f64) / 100.0)
    } else {
        format!("${:.2}", (cents as f64) / 100.0)
    }
}
