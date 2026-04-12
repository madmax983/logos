#[must_use]
pub fn us_timestamp(us: i64) -> String {
    chrono::DateTime::from_timestamp_micros(us).map_or_else(
        || us.to_string(),
        |dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
    )
}
