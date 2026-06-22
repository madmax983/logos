#![allow(clippy::should_panic_without_expect)]
use logos_core::fire::FireSimulator;
use proptest::prelude::*;

proptest! {
    #[test]
    #[should_panic(expected = "attempt to add with overflow")]
    fn havoc_add_assets_liabilities_panics_on_overflow(
        assets in (i64::MAX / 2 + 1)..=i64::MAX,
    ) {
        let mut sim = FireSimulator::new(5000);
        sim.add_assets_liabilities(assets, 0);
        sim.add_assets_liabilities(assets, 0);
    }
}
