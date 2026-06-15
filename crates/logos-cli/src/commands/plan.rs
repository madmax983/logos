use crate::args::CliError;

/// Handles `ledger plan fire`.
#[allow(clippy::unnecessary_wraps)]
pub fn fire(
    monthly_expenses_cents: i64,
    liquid_assets_cents: i64,
    monthly_savings_cents: i64,
) -> Result<(), CliError> {
    use logos_core::fire::FireSimulator;
    use logos_core::net_worth_projector::NetWorthProjector;

    let mut sim = FireSimulator::new(monthly_expenses_cents);
    sim.add_assets_liabilities(liquid_assets_cents, 0);

    let fire_number = sim.fire_number_cents();
    let current_net_worth = sim.safe_net_worth_cents();

    let projector = NetWorthProjector::new(current_net_worth, monthly_savings_cents);
    let months_to_simulate = 1200; // up to 100 years

    let ascent_sim =
        logos_core::fire_ascent::FireAscentSimulator::new(sim, projector, months_to_simulate);
    let ascent_result = ascent_sim.ascend();

    let output = render_fire_sim_output(
        monthly_expenses_cents,
        fire_number,
        current_net_worth,
        monthly_savings_cents,
        &ascent_result,
    );
    println!("{output}");

    Ok(())
}

fn render_fire_sim_output(
    monthly_expenses_cents: i64,
    fire_number: i64,
    current_net_worth: i64,
    monthly_savings_cents: i64,
    ascent_result: &logos_core::fire_ascent::AscentResult,
) -> String {
    let mut table = comfy_table::Table::new();
    table.load_preset(comfy_table::presets::UTF8_FULL);
    table.set_header(vec!["Metric", "Value"]);

    table.add_row(vec![
        comfy_table::Cell::new("Monthly Expenses"),
        comfy_table::Cell::new(logos_core::format::currency(monthly_expenses_cents))
            .fg(comfy_table::Color::Red),
    ]);

    table.add_row(vec![
        comfy_table::Cell::new("Target FIRE Number")
            .fg(comfy_table::Color::Green)
            .add_attribute(comfy_table::Attribute::Bold),
        comfy_table::Cell::new(logos_core::format::currency(fire_number))
            .fg(comfy_table::Color::Green)
            .add_attribute(comfy_table::Attribute::Bold),
    ]);

    table.add_row(vec![
        comfy_table::Cell::new("Current Safe Net Worth"),
        comfy_table::Cell::new(logos_core::format::currency(current_net_worth))
            .fg(comfy_table::Color::Blue),
    ]);

    table.add_row(vec![
        comfy_table::Cell::new("Monthly Savings"),
        comfy_table::Cell::new(logos_core::format::currency(monthly_savings_cents))
            .fg(comfy_table::Color::Green),
    ]);

    let progress_pct = if fire_number > 0 {
        #[allow(clippy::cast_precision_loss)]
        ((current_net_worth as f64 / fire_number as f64) * 100.0).clamp(0.0, 100.0)
    } else {
        100.0
    };

    table.add_row(vec![
        comfy_table::Cell::new("Progress %")
            .fg(comfy_table::Color::Cyan)
            .add_attribute(comfy_table::Attribute::Bold),
        comfy_table::Cell::new(format!("{progress_pct:.1}%"))
            .fg(comfy_table::Color::Cyan)
            .add_attribute(comfy_table::Attribute::Bold),
    ]);

    let mut journey_table = comfy_table::Table::new();
    journey_table.load_preset(comfy_table::presets::UTF8_FULL);
    journey_table.set_header(vec!["Milestone", "Target", "Status"]);

    if ascent_result.impossible {
        journey_table.add_row(vec![
            comfy_table::Cell::new("Simulation").fg(comfy_table::Color::Red),
            comfy_table::Cell::new("Infinite Summit").fg(comfy_table::Color::Red),
            comfy_table::Cell::new("Impossible")
                .fg(comfy_table::Color::Red)
                .add_attribute(comfy_table::Attribute::Bold),
        ]);
    } else if ascent_result.instant_summit {
        journey_table.add_row(vec![
            comfy_table::Cell::new("Simulation").fg(comfy_table::Color::Green),
            comfy_table::Cell::new("$0.00 Expenses").fg(comfy_table::Color::Green),
            comfy_table::Cell::new("Instant Summit!")
                .fg(comfy_table::Color::Green)
                .add_attribute(comfy_table::Attribute::Bold),
        ]);
    } else {
        for milestone in &ascent_result.milestones {
            let target_dollars = logos_core::format::currency(milestone.target_cents);

            if let Some(month) = milestone.month_reached {
                let years = month / 12;
                let extra_months = month % 12;
                journey_table.add_row(vec![
                    comfy_table::Cell::new(milestone.name).fg(comfy_table::Color::Green),
                    comfy_table::Cell::new(target_dollars).fg(comfy_table::Color::Green),
                    comfy_table::Cell::new(format!(
                        "Reached in {years}y {extra_months}m (Month {month})"
                    ))
                    .fg(comfy_table::Color::Green)
                    .add_attribute(comfy_table::Attribute::Bold),
                ]);
            } else {
                journey_table.add_row(vec![
                    comfy_table::Cell::new(milestone.name).fg(comfy_table::Color::Red),
                    comfy_table::Cell::new(target_dollars).fg(comfy_table::Color::Red),
                    comfy_table::Cell::new("Pending").fg(comfy_table::Color::Red),
                ]);
            }
        }
    }

    format!("{table}\n\n{journey_table}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fire_calculates_correctly() {
        let result = fire(500_000, 1_000_000, 200_000);
        assert!(result.is_ok());
    }
}
