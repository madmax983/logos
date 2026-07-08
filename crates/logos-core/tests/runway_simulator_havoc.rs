#![cfg(feature = "nova")]
#![allow(clippy::should_panic_without_expect)]

#[test]
#[should_panic]
fn havoc_runway_simulator_panics_on_overflow() {
    let sim = logos_core::runway_simulator::RunwaySimulator::new(i64::MAX, i64::MAX, f64::MAX);
    let _ = sim.calculate_runway();
}
