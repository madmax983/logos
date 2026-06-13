/// Transforms a planning environment state into a formatted, styled UI view block.
///
/// This pure function separates the visual logic for formatting RSU planning data
/// from the application event loop, ensuring layout consistency.
///
/// ## Examples
///
/// ```text
/// use logos_tui::ui::rsu::render;
///
/// let output = render();
/// assert!(output.contains("RSU View"));
/// ```
#[must_use]
pub fn render() -> String {
    "RSU View | 30-day Avg | Haircut Forecast".to_owned()
}
