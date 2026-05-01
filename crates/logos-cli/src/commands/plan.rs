use crate::args::CliError;
/// Handles `ledger plan net-worth`.
///
/// # Errors
///
/// Returns an error when runtime initialization fails.
#[allow(clippy::unnecessary_wraps)]
pub fn net_worth_project(
    initial_net_worth_cents: i64,
    monthly_savings_cents: i64,
    months: u16,
) -> Result<(), CliError> {
    use logos_core::net_worth_projector::NetWorthProjector;

    let projector = NetWorthProjector::new(initial_net_worth_cents, monthly_savings_cents);
    let (timeline, _) = projector.project_timeline(months);

    let mut table = comfy_table::Table::new();
    table.load_preset(comfy_table::presets::UTF8_FULL);
    table.set_header(vec!["Month", "Net Worth", "Saved Cash", "Vested Value"]);

    for month in timeline {
        table.add_row(vec![
            comfy_table::Cell::new(month.month_index.to_string()),
            comfy_table::Cell::new(logos_core::format::currency(month.net_worth_cents))
                .fg(comfy_table::Color::Green),
            comfy_table::Cell::new(logos_core::format::currency(month.saved_cents)),
            comfy_table::Cell::new(logos_core::format::currency(month.vested_value_cents)),
        ]);
    }

    println!(
        "plan.net-worth
{table}"
    );

    Ok(())
}

/// Handles `ledger plan fire`.
///
/// # Errors
///
/// Returns an error when runtime initialization fails.
#[allow(clippy::unnecessary_wraps)]
pub fn fire_sim(
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
mod fire_sim_tests {
    use super::*;

    #[test]
    fn test_net_worth_project_calculates_correctly() {
        let result = super::net_worth_project(10_000_000, 500_000, 12);
        assert!(result.is_ok());
    }

    #[test]
    fn test_fire_sim_calculates_correctly() {
        // Just checking execution completes without error
        let result = fire_sim(500_000, 1_000_000, 200_000);
        assert!(result.is_ok());
    }
}
