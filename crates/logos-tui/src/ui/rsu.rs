#[must_use]
pub fn render() -> String {
    "RSU View | 30-day Avg | Haircut Forecast".to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rsu_render() {
        let result = render();
        assert!(result.contains("RSU View"));
        assert!(result.contains("30-day Avg"));
        assert!(result.contains("Haircut Forecast"));
    }
}
