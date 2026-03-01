use logos_core::domain::rsu::{AllocationPolicy, HaircutTierTable, forecast_value_cents};

#[test]
fn test_haircut_tier_table() {
    let table = HaircutTierTable::conservative_defaults();
    assert_eq!(table.haircut_for_days(29), 25);
    assert_eq!(table.haircut_for_days(30), 40); // Test the < 30 boundary
    assert_eq!(table.haircut_for_days(90), 40);
    assert_eq!(table.haircut_for_days(91), 55); // Test the <= 90 boundary
}

#[test]
fn test_allocation_policy() {
    let policy = AllocationPolicy::new(30, 20, 40, 10).unwrap();
    assert_eq!(policy.tax_reserve_pct(), 30);
    assert_eq!(policy.smoothing_buffer_pct(), 20);
    assert_eq!(policy.goals_pct(), 40);
    assert_eq!(policy.discretionary_pct(), 10);
}

#[test]
fn test_forecast_value_cents() {
    let table = HaircutTierTable::conservative_defaults();
    // 30 days -> 40% haircut -> 60% retained
    // 10 units @ 10000 cents = 100000 cents
    // 60% of 100000 = 60000
    assert_eq!(forecast_value_cents(10000, 10, 30, &table), 60000);

    // 0 forecast value
    assert_eq!(forecast_value_cents(0, 10, 30, &table), 0);
}
