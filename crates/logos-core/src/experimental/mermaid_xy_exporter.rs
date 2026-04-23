use crate::planning::net_worth_projector::ProjectedMonth;
use std::fmt::Write;

/// Exports a projection timeline into a Mermaid XY Chart.
///
/// This provides a visual representation of net worth growth over time.
#[derive(Debug, Default)]
pub struct MermaidXyExporter {}

impl MermaidXyExporter {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Generates a Mermaid XY Chart representing net worth growth.
    #[must_use]
    pub fn export_net_worth_xy(&self, timeline: &[ProjectedMonth]) -> String {
        if timeline.is_empty() {
            return String::from("```mermaid\nxychart-beta\n```\n");
        }

        let mut output = String::from("```mermaid\nxychart-beta\n");
        let _ = writeln!(output, "    title \"Net Worth Projection\"");

        let mut x_axis_str = String::with_capacity(timeline.len() * 4);
        let mut y_str = String::with_capacity(timeline.len() * 7);
        let mut min_nw = i64::MAX;
        let mut max_nw = i64::MIN;

        for (i, month) in timeline.iter().enumerate() {
            if i > 0 {
                x_axis_str.push_str(", ");
                y_str.push_str(", ");
            }
            let _ = write!(x_axis_str, "{}", month.month_index);

            let nw_dollars = month.net_worth_cents / 100;
            let _ = write!(y_str, "{nw_dollars}");

            if nw_dollars < min_nw {
                min_nw = nw_dollars;
            }
            if nw_dollars > max_nw {
                max_nw = nw_dollars;
            }
        }

        let _ = writeln!(output, "    x-axis \"Month\" [{x_axis_str}]");

        // Give a little padding for the y-axis
        let y_min = if min_nw < 0 { min_nw } else { 0 };
        let y_max = if max_nw < 0 { 0 } else { max_nw };

        let _ = writeln!(output, "    y-axis \"Net Worth ($)\" {y_min} --> {y_max}");
        let _ = writeln!(output, "    line [{y_str}]");
        output.push_str("```\n");

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_empty_timeline() {
        let exporter = MermaidXyExporter::new();
        assert_eq!(
            exporter.export_net_worth_xy(&[]),
            "```mermaid\nxychart-beta\n```\n"
        );
    }

    #[test]
    fn test_export_net_worth_timeline() {
        let exporter = MermaidXyExporter::new();
        let timeline = vec![
            ProjectedMonth {
                month_index: 1,
                net_worth_cents: 100_000,
                vested_value_cents: 0,
                saved_cents: 100_000,
            },
            ProjectedMonth {
                month_index: 2,
                net_worth_cents: 200_000,
                vested_value_cents: 0,
                saved_cents: 100_000,
            },
            ProjectedMonth {
                month_index: 3,
                net_worth_cents: 350_000,
                vested_value_cents: 50_000,
                saved_cents: 100_000,
            },
        ];

        let expected = "\
```mermaid
xychart-beta
    title \"Net Worth Projection\"
    x-axis \"Month\" [1, 2, 3]
    y-axis \"Net Worth ($)\" 0 --> 3500
    line [1000, 2000, 3500]
```
";
        assert_eq!(exporter.export_net_worth_xy(&timeline), expected);
    }
}
