use comfy_table::Table;

use crate::app::HomeSnapshot;

#[must_use]
pub fn render(
    month_key: &str,
    checking_account: &str,
    expense_account_prefix: &str,
    snapshot: Option<&HomeSnapshot>,
) -> String {
    let mut lines = vec![
        "Logos Home Dashboard".to_owned(),
        format!(
            "Month: {month_key} | Checking: {checking_account} | Expense Prefix: {expense_account_prefix}"
        ),
        String::new(),
    ];

    let Some(snapshot) = snapshot else {
        lines.push("(no dashboard data loaded)".to_owned());
        return lines.join("\n");
    };

    let mut month_table = Table::new();
    month_table.set_header(vec!["Metric", "Value (Cents)"]);
    month_table.add_row(vec![
        "Checking Balance".to_owned(),
        snapshot.checking_balance_cents().to_string(),
    ]);
    month_table.add_row(vec![
        "Income".to_owned(),
        snapshot.income_cents().to_string(),
    ]);
    month_table.add_row(vec![
        "Expense".to_owned(),
        snapshot.expense_cents().to_string(),
    ]);
    month_table.add_row(vec![
        "Cashflow".to_owned(),
        snapshot.cashflow_cents().to_string(),
    ]);
    lines.push(month_table.to_string());
    lines.push(String::new());

    let mut budget_table = Table::new();
    budget_table.set_header(vec!["Budget", "Value"]);
    budget_table.add_row(vec!["Month".to_owned(), snapshot.month_key().to_owned()]);
    budget_table.add_row(vec![
        "Checking Account".to_owned(),
        snapshot.checking_account().to_owned(),
    ]);
    budget_table.add_row(vec![
        "Expense Prefix".to_owned(),
        snapshot.expense_account_prefix().to_owned(),
    ]);
    budget_table.add_row(vec![
        "Target (Cents)".to_owned(),
        snapshot
            .budget_target_cents()
            .map_or_else(|| "unconfigured".to_owned(), |value| value.to_string()),
    ]);
    budget_table.add_row(vec![
        "Variance (Cents)".to_owned(),
        snapshot
            .budget_variance_cents()
            .map_or_else(|| "unconfigured".to_owned(), |value| value.to_string()),
    ]);
    lines.push(budget_table.to_string());

    lines.join("\n")
}
