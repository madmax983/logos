use crate::args::CliError;
use comfy_table::{Attribute, Cell, Color, Table, presets::UTF8_FULL};
use logos_core::fire::{FireConfig, FireSimulator};
use logos_core::format::currency;
use logos_core::net_worth_projector::NetWorthProjector;

#[allow(clippy::unnecessary_wraps)]
pub fn fire(
    monthly_expenses_cents: i64,
    safe_withdrawal_rate_pct: Option<u8>,
    liquid_assets_cents: Option<i64>,
    liabilities_cents: Option<i64>,
) -> Result<(), CliError> {
    let mut sim = FireSimulator::new(monthly_expenses_cents);
    if let Some(rate) = safe_withdrawal_rate_pct {
        sim.set_config(FireConfig {
            safe_withdrawal_rate_pct: rate,
        });
    }

    let assets = liquid_assets_cents.unwrap_or(0);
    let liab = liabilities_cents.unwrap_or(0);
    sim.add_assets_liabilities(assets, liab);

    let fire_number = sim.fire_number_cents();
    let safe_nw = sim.safe_net_worth_cents();
    let progress = sim.fire_progress_pct();

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec!["Metric", "Value"]);

    table.add_row(vec![
        Cell::new("Monthly Expenses"),
        Cell::new(currency(monthly_expenses_cents)).fg(Color::Red),
    ]);

    table.add_row(vec![
        Cell::new("Target FIRE Number")
            .fg(Color::Green)
            .add_attribute(Attribute::Bold),
        Cell::new(currency(fire_number))
            .fg(Color::Green)
            .add_attribute(Attribute::Bold),
    ]);

    table.add_row(vec![
        Cell::new("Safe Net Worth"),
        Cell::new(currency(safe_nw)).fg(Color::Blue),
    ]);

    table.add_row(vec![
        Cell::new("Progress"),
        Cell::new(format!("{progress}%")).fg(Color::Yellow),
    ]);

    println!("{table}");
    Ok(())
}

#[allow(clippy::unnecessary_wraps)]
pub fn project(
    initial_net_worth_cents: i64,
    monthly_savings_cents: i64,
    months: u16,
) -> Result<(), CliError> {
    let projector = NetWorthProjector::new(initial_net_worth_cents, monthly_savings_cents);
    let (timeline, _) = projector.project_timeline(months);

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec!["Month", "Net Worth", "Saved Cash", "Vested Value"]);

    for month in timeline {
        table.add_row(vec![
            Cell::new(month.month_index.to_string()),
            Cell::new(currency(month.net_worth_cents)).fg(Color::Green),
            Cell::new(currency(month.saved_cents)),
            Cell::new(currency(month.vested_value_cents)),
        ]);
    }

    println!("Net Worth Projection\n{table}");

    Ok(())
}
