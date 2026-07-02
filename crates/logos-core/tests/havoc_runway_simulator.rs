#![cfg(feature = "nova")]
#![allow(clippy::should_panic_without_expect)]

use logos_core::runway_simulator::RunwaySimulator;
use proptest::prelude::*;

proptest! {
    #[test]
    #[should_panic(expected = "attempt to subtract with overflow")]
    fn havoc_runway_simulator_panics_on_overflow(
        burn in i64::MIN..=i64::MIN + 1000,
    ) {
        // Force assets to be zero or normal, but burn is large negative.
        // current_assets.min(burn) will be burn (large negative).
        // current_assets -= actual_burn -> current_assets - (-MAX) -> overflow!
        let sim = RunwaySimulator::new(100, burn, 5.0);
        let _ = sim.calculate_runway();
    }
}
