#[must_use]
pub const fn project_net_worth(assets_cents: i64, liabilities_cents: i64) -> i64 {
    assets_cents - liabilities_cents
}
