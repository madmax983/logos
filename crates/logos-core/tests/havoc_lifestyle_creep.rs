#![allow(clippy::should_panic_without_expect)]
// Test is conditionally compiled since `lifestyle_creep` is behind the `nova` feature.
#![cfg(feature = "nova")]

use logos_core::fire::FireSimulator;
use logos_core::lifestyle_creep::LifestyleCreepSimulator;
use proptest::prelude::*;

proptest! {
    #[test]
    #[should_panic]
    fn project_creep_panics_on_overflow(
        net_worth in 1_i64..=1_000,
        income in i64::MIN..=-1_000_000,
        expenses in 1_000_000_i64..=i64::MAX,
    ) {
        let sim = LifestyleCreepSimulator::new(
            FireSimulator::new(expenses),
            net_worth,
            income,
            0.0,
            0.0,
            0.0,
        );
        let _ = sim.calculate();
    }
}
