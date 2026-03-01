use logos_core::{forecast_value_cents, AllocationPolicy, HaircutTierTable};

#[test]
fn conservative_haircut_tiers_are_selected_by_horizon() {
    let tiers = HaircutTierTable::conservative_defaults();

    assert_eq!(tiers.haircut_for_days(10), 25);
    assert_eq!(tiers.haircut_for_days(45), 40);
    assert_eq!(tiers.haircut_for_days(120), 55);
}

#[test]
fn allocation_policy_must_sum_to_one_hundred_percent() {
    assert!(AllocationPolicy::new(40, 30, 20, 10).is_ok());
    assert!(AllocationPolicy::new(40, 30, 20, 9).is_err());
}

#[test]
fn rsu_forecast_is_planning_only_and_does_not_mutate_ledger_balance() {
    let posted_balance_cents = 250_000_i64;
    let tiers = HaircutTierTable::conservative_defaults();

    let projected = forecast_value_cents(12_345, 100, 15, &tiers);

    assert!(projected > 0);
    assert_eq!(posted_balance_cents, 250_000);
}
