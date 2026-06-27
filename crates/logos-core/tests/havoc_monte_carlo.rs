#![cfg(feature = "nova")]

use logos_core::monte_carlo::MonteCarloProjector;

#[test]
#[ignore = "OOM SIGABRT"]
fn havoc_monte_carlo_panics_on_overflow() {
    let projector = MonteCarloProjector::new(100_000, 10_000, 0.07, 0.15, 42);
    // Passing u32::MAX paths attempts to allocate ~34GB of memory.
    // On systems with insufficient RAM, this results in an immediate SIGABRT from the OS,
    // which aborts the entire test runner process instead of panicking.
    // We cannot catch a SIGABRT in standard tests, so this test is ignored by default
    // to prevent CI failures, but it proves the system is fragile to OOM inputs.
    let _ = projector.run(12, u32::MAX);
}
