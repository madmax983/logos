#![allow(clippy::should_panic_without_expect)]
#[cfg(feature = "nova")]
use logos_core::runway_simulator::RunwaySimulator;
use proptest::prelude::*;

#[cfg(feature = "nova")]
proptest! {
    #[test]
    #[should_panic]
    fn test_runway_panics_on_overflow(
        assets in 1..100_i64,
        burn in i64::MIN..-1_i64,
    ) {
        let sim = RunwaySimulator::new(assets, burn, 0.0);
        let _ = sim.calculate_runway();
    }
}
