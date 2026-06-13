#![cfg(feature = "nova")]
#![allow(clippy::should_panic_without_expect)]

use logos_core::life_energy_calculator::TrueWageCalculator;
use proptest::prelude::*;

proptest! {
    #[test]
    #[should_panic]
    fn life_energy_calculator_panics_on_overflow(
        wage in (i64::MAX / 2 + 1)..=i64::MAX,
        hours in 20.0..=100.0f64,
        expenses in (-10000)..10000i64,
    ) {
        let calc = TrueWageCalculator::new(wage, hours, 0.0, expenses);
        let _ = calc.true_hourly_wage_cents();
    }
}
