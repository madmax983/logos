#![allow(clippy::should_panic_without_expect)]
use logos_core::NetWorthProjector;
use proptest::prelude::*;

proptest! {
    #[test]
    #[should_panic(expected = "attempt to multiply with overflow")]
    fn project_timeline_panics_on_multiplication_overflow(
        months in (u16::MAX / 30 + 1)..=u16::MAX,
    ) {
        let projector = NetWorthProjector::new(100_000, 10_000);
        let _ = projector.project_timeline(months);
    }
}
