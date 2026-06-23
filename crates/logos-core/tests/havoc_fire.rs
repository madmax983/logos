#![allow(clippy::should_panic_without_expect)]
use logos_core::fire::FireSimulator;
use proptest::prelude::*;

proptest! {
    #[test]
    #[should_panic]
    fn test_add_assets_liabilities_panics_on_overflow(
        amount in (i64::MAX / 2 + 1)..=i64::MAX,
    ) {
        let mut sim = FireSimulator::new(500_000);
        sim.add_assets_liabilities(amount, 0);
        sim.add_assets_liabilities(amount, 0);
    }
}
