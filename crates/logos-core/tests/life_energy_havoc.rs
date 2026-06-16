#![allow(clippy::should_panic_without_expect)]

#[cfg(feature = "nova")]
#[test]
#[should_panic]
fn test_life_energy_panics_on_overflow() {
    use logos_core::life_energy_calculator::TrueWageCalculator;
    let calc = TrueWageCalculator::new(
        i64::MAX, // nominal_hourly_wage_cents
        1.0,      // weekly_hours_worked
        0.0,      // weekly_commute_hours
        -100,     // weekly_job_expenses_cents
    );
    let _ = calc.true_hourly_wage_cents();
}
