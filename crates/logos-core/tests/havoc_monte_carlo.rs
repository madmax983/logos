#![cfg(feature = "nova")]
use logos_core::monte_carlo::MonteCarloProjector;

#[test]
fn havoc_monte_carlo_prevents_oom_with_cap() {
    let projector = MonteCarloProjector::new(10_000, 10_000, 0.07, 0.15, 42);
    // When passed a huge number of paths (like u32::MAX), Vec::with_capacity tries to allocate
    // 34+ GB of memory, causing the allocator to panic with "capacity overflow"
    let _ = projector.run(12, u32::MAX);
}
