use comfy_table::{Cell, Color, Table};

use crate::app::RegisterSnapshot;
use logos_cli::format::currency;

#[must_use]
pub fn render(account: &str, snapshot: Option<&RegisterSnapshot>) -> String {
    let mut lines = vec![
        String::from("Register View"),
        format!("Account: {account}"),
        String::new(),
    ];

    let Some(snapshot) = snapshot else {
        lines.push(String::from("(register data unavailable)"));
        return lines.join("\n");
    };

    lines.push(format!("Balance: {}", currency(snapshot.balance_cents())));
    lines.push(String::new());
    lines.push(String::from("Recent Activity:"));

    if snapshot.activity().is_empty() {
        lines.push(String::from("  (no activity for selected account)"));
        return lines.join("\n");
    }

    let mut table = Table::new();
    table.set_header(vec!["Timestamp", "Description", "Amount"]);
    for record in snapshot.activity() {
        let amount_cell = match record.amount_cents().cmp(&0) {
            std::cmp::Ordering::Greater => {
                Cell::new(currency(record.amount_cents())).fg(Color::Green)
            }
            std::cmp::Ordering::Less => Cell::new(currency(record.amount_cents())).fg(Color::Red),
            std::cmp::Ordering::Equal => Cell::new(currency(record.amount_cents())),
        };
        table.add_row(vec![
            Cell::new(record.timestamp()),
            Cell::new(record.description()),
            amount_cell,
        ]);
    }
    lines.push(table.to_string());

    lines.join("\n")
}
