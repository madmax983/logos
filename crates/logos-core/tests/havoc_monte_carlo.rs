#![cfg(feature = "nova")]
use logos_core::monte_carlo::MonteCarloProjector;

#[test]
fn havoc_monte_carlo_panics_on_oom() {
    let projector = MonteCarloProjector::new(10_000, 10_000, 0.07, 0.15, 42);
    // We now clamp to 1_000_000, so this should not panic.
    let _ = projector.run(12, u32::MAX);
}
