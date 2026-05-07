#![allow(unused)]
#[cfg(feature = "nova")]
use logos_core::monte_carlo::MonteCarloProjector;

#[test]
#[should_panic(expected = "memory allocation")]
#[ignore = "OOM"]
fn havoc_monte_carlo_allocation_crash() {
    #[cfg(feature = "nova")]
    {
        let projector = MonteCarloProjector::new(100_000, 1000, 0.07, 0.15, 42);
        // Huge paths will cause OOM
        let _ = projector.run(12, 4_294_967_295);
    }
}
