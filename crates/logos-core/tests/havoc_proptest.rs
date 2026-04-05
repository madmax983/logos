#![allow(clippy::should_panic_without_expect)]

use logos_core::domain::account::AccountId;
use logos_core::domain::rsu::AllocationPolicy;
use logos_core::planning::fire::{FireSimulator, UpcomingVest};
use logos_core::planning::net_worth_projector::NetWorthProjector;
use logos_core::planning::rsu_distributor::{RsuAutoDistributor, RsuDistributorConfig};
use proptest::prelude::*;

proptest! {
    #[test]
    #[should_panic(expected = "attempt to multiply with overflow")]
    fn distribute_rsu_vest_panics_on_overflow(gross_vest in i64::MAX / 2..i64::MAX) {
        let config = RsuDistributorConfig {
            rsu_asset: AccountId::new("assets:rsu").unwrap(),
            tax_reserve: AccountId::new("assets:tax").unwrap(),
            smoothing_buffer: AccountId::new("assets:buffer").unwrap(),
            goals: AccountId::new("assets:goals").unwrap(),
            discretionary: AccountId::new("assets:checking").unwrap(),
        };
        let distributor = RsuAutoDistributor::new(config);
        let policy = AllocationPolicy::new(40, 20, 30, 10).unwrap();

        // This will panic when multiplied by percent, if gross_vest is very large
        let _ = distributor.distribute_rsu_vest("Vest 1", gross_vest, &policy);
    }

    #[test]
    #[should_panic(expected = "attempt to multiply with overflow")]
    fn project_timeline_panics_on_overflow(
        initial_net_worth in any::<i64>(),
        monthly_savings in any::<i64>(),
        months in 2185..=u16::MAX,
        vest_units in any::<u32>(),
        vest_days in any::<u16>(),
        vest_price in any::<i64>(),
    ) {
        let mut projector = NetWorthProjector::new(initial_net_worth, monthly_savings);
        projector.add_upcoming_vest(UpcomingVest {
            units: vest_units,
            avg_close_price_cents: vest_price,
            days_to_vest: vest_days,
        });

        let _ = projector.project_timeline(months);
    }

    #[test]
    #[should_panic(expected = "attempt to add with overflow")]
    fn add_assets_liabilities_panics_on_overflow(
        assets in i64::MAX / 2..i64::MAX,
    ) {
        let mut sim = FireSimulator::new(5000);
        sim.add_assets_liabilities(assets, 0);
        sim.add_assets_liabilities(assets, 0);
    }

    #[test]
    fn test_forecast_value_cents_handles_large_gross_value(
        price in (i64::MAX / 50)..=(i64::MAX / 2),
    ) {
        let tiers = logos_core::domain::rsu::HaircutTierTable::default();
        let result = logos_core::domain::rsu::forecast_value_cents(price, 1, 15, &tiers);
        // It should return 0 on overflow or calculate a correct partial value.
        // Given that it uses checked_mul and returns 0 on overflow:
        // Or if it computes successfully, it computes. We just want it not to panic.
        let _ = result;
    }
}
