#[must_use]
pub fn render() -> String {
    "RSU View | 30-day Avg | Haircut Forecast".to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_is_deterministic() {
        let output = render();
        assert!(output.contains("RSU View"));
        assert!(output.contains("30-day Avg"));
        assert!(output.contains("Haircut Forecast"));
    }
}
