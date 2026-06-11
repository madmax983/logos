use logos_core::forecast_value_cents;
use logos_core::HaircutTierTable;
use proptest::prelude::*;

proptest! {
    #[test]
    fn havoc_rsu_forecast_value_cents(
        price in any::<i64>(),
        units in any::<u32>(),
        days in any::<u16>(),
    ) {
        let tiers = HaircutTierTable::conservative_defaults();
        let _ = forecast_value_cents(price, units, days, &tiers);
    }
}
