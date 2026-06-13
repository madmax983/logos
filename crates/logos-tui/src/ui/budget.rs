use comfy_table::Table;

use crate::app::BudgetSnapshot;
use logos_core::format::currency;

/// Transforms a budget summary state into a formatted, styled UI view block.
///
/// This pure function separates the visual logic for formatting budget tracking
/// from the application event loop, ensuring layout consistency.
///
/// ## Examples
///
/// ```text
/// use logos_tui::{ui::budget::render, BudgetSnapshot};
///
/// let snapshot = BudgetSnapshot::new("2026-03", "expenses:", Some(3000_00), 1500_00, Some(-1500_00));
/// let output = render("2026-03", "expenses:", Some(&snapshot));
/// assert!(output.contains("Budget View"));
/// ```
#[must_use]
pub fn render(
    month_key: &str,
    expense_account_prefix: &str,
    snapshot: Option<&BudgetSnapshot>,
) -> String {
    let mut lines = vec![
        "Budget View".to_owned(),
        format!("Month: {month_key} | Expense Prefix: {expense_account_prefix}"),
        String::new(),
    ];

    let Some(snapshot) = snapshot else {
        lines.push("(no budget data loaded)".to_owned());
        return lines.join("\n");
    };

    let mut table = Table::new();
    table.set_header(vec!["Field", "Value"]);
    table.add_row(vec!["Month".to_owned(), snapshot.month_key().to_owned()]);
    table.add_row(vec![
        "Expense Prefix".to_owned(),
        snapshot.expense_account_prefix().to_owned(),
    ]);
    table.add_row(vec![
        "Target".to_owned(),
        snapshot
            .budget_target_cents()
            .map_or_else(|| "unconfigured".to_owned(), currency),
    ]);
    table.add_row(vec![
        "Actual Expense".to_owned(),
        currency(snapshot.actual_expense_cents()),
    ]);
    table.add_row(vec![
        "Variance".to_owned(),
        snapshot
            .budget_variance_cents()
            .map_or_else(|| "unconfigured".to_owned(), currency),
    ]);
    lines.push(table.to_string());

    lines.join("\n")
}
