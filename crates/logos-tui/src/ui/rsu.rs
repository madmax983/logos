#[must_use]
pub fn render() -> String {
    "RSU View | 30-day Avg | Haircut Forecast".to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rsu_view_renders_placeholder() {
        assert_eq!(render(), "RSU View | 30-day Avg | Haircut Forecast");
    }
}
