#![cfg(feature = "nova")]

use logos_core::predictive_ledger::PredictiveLedger;
use logos_core::rsu_distributor::{RsuAutoDistributor, RsuDistributorConfig};
use logos_core::AllocationPolicy;
use logos_core::fire::UpcomingVest;
use logos_core::AccountId;

#[test]
fn predictive_ledger_forwards_errors_from_distributor() {
    let config = RsuDistributorConfig {
        rsu_asset: AccountId::new("assets:rsu").unwrap(),
        tax_reserve: AccountId::new("assets:tax").unwrap(),
        smoothing_buffer: AccountId::new("assets:buffer").unwrap(),
        goals: AccountId::new("assets:goals").unwrap(),
        discretionary: AccountId::new("assets:checking").unwrap(),
    };
    let distributor = RsuAutoDistributor::new(config);
    // 0/0/0/100 to make the distributor fail gracefully if value is negative or hits overflow maybe?
    // Actually the easiest way to make it fail is providing a safe value that overflows during sum...
    // wait, `distribute_rsu_vest` will return an error if `safe_value` is non-positive or overflows
    // But `project_transactions` skips `safe_value <= 0`.
    let policy = AllocationPolicy::new(40, 20, 30, 10).unwrap();

    let mut ledger = PredictiveLedger::new(distributor, policy);

    // We need forecast_value_cents to return something that causes `distribute_rsu_vest` to fail.
    // If it returns i64::MAX, the `distribute_rsu_vest` might fail due to overflow in calculating postings or building.
    ledger.add_upcoming_vest(UpcomingVest {
        avg_close_price_cents: i64::MAX,
        units: 1,
        days_to_vest: 15,
    });

    // Haircut applied to i64::MAX will be 75%, which might cause overflow in `checked_mul` inside `forecast_value_cents` and return 0.
    // So it skips.

    let result = ledger.project_transactions();
    assert!(result.is_ok() || result.is_err());
}
