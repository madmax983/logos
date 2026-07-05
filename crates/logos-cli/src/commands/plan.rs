use crate::args::CliError;
use crossterm::style::Stylize;
use logos_core::fire::{FireSimulator, UpcomingVest};

/// Handles `ledger plan fire`.
///
/// # Errors
///
/// Returns an error when execution fails.
#[allow(clippy::unnecessary_wraps)]
pub fn fire(
    monthly_expenses_cents: i64,
    liquid_assets_cents: i64,
    liabilities_cents: i64,
    upcoming_vests_cents: i64,
) -> Result<(), CliError> {
    let mut sim = FireSimulator::new(monthly_expenses_cents);
    sim.add_assets_liabilities(liquid_assets_cents, liabilities_cents);

    if upcoming_vests_cents > 0 {
        sim.add_upcoming_vest(UpcomingVest {
            avg_close_price_cents: upcoming_vests_cents,
            units: 1,
            days_to_vest: 0,
        });
    }

    let fire_number = sim.fire_number_cents();
    let current_net_worth = sim.safe_net_worth_cents();
    let progress = sim.fire_progress_pct();

    println!("\n{}\n", "🔥 FIRE Projections Dashboard".green().bold());

    let mut input_table = comfy_table::Table::new();
    input_table.load_preset(comfy_table::presets::UTF8_FULL);
    input_table.set_header(vec![
        comfy_table::Cell::new("Input Param").add_attribute(comfy_table::Attribute::Bold),
        comfy_table::Cell::new("Value").add_attribute(comfy_table::Attribute::Bold),
    ]);
    input_table.add_row(vec![
        "Monthly Expenses",
        &logos_core::format::currency(monthly_expenses_cents),
    ]);
    input_table.add_row(vec![
        "Liquid Assets",
        &logos_core::format::currency(liquid_assets_cents),
    ]);
    input_table.add_row(vec![
        "Liabilities",
        &logos_core::format::currency(liabilities_cents),
    ]);
    input_table.add_row(vec![
        "Upcoming Vests",
        &logos_core::format::currency(upcoming_vests_cents),
    ]);

    println!("{input_table}\n");

    let mut summary_table = comfy_table::Table::new();
    summary_table.load_preset(comfy_table::presets::UTF8_FULL);
    summary_table.set_header(vec![
        comfy_table::Cell::new("Target FIRE Number")
            .fg(comfy_table::Color::Green)
            .add_attribute(comfy_table::Attribute::Bold),
        comfy_table::Cell::new("Safe Net Worth")
            .fg(comfy_table::Color::Blue)
            .add_attribute(comfy_table::Attribute::Bold),
        comfy_table::Cell::new("Progress %")
            .fg(comfy_table::Color::Yellow)
            .add_attribute(comfy_table::Attribute::Bold),
    ]);

    let progress_str = format!("{progress}%");
    summary_table.add_row(vec![
        comfy_table::Cell::new(logos_core::format::currency(fire_number))
            .fg(comfy_table::Color::Green)
            .add_attribute(comfy_table::Attribute::Bold),
        comfy_table::Cell::new(logos_core::format::currency(current_net_worth))
            .fg(comfy_table::Color::Blue),
        comfy_table::Cell::new(progress_str).fg(comfy_table::Color::Yellow),
    ]);

    println!("{summary_table}\n");

    if progress >= 100 {
        println!(
            "{}\n",
            "🎉 Congratulations! You have reached your FIRE goal!"
                .green()
                .bold()
        );
    } else {
        println!(
            "{}\n",
            "Keep up the great work on your FIRE journey! 🚀".cyan()
        );
    }

    Ok(())
}
