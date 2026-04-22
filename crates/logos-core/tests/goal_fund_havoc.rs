#![allow(clippy::should_panic_without_expect)]

#[cfg(feature = "nova")]
use logos_core::AllocationPolicy;
#[cfg(feature = "nova")]
use logos_core::experimental::goal_fund_projector::GoalFundProjector;
#[cfg(feature = "nova")]
use proptest::prelude::*;

#[cfg(feature = "nova")]
proptest! {
    #[test]
    #[should_panic(expected = "attempt to multiply with overflow")]
    fn project_timeline_panics_on_u16_overflow(
        months in 3000..=u16::MAX,
    ) {
        let policy = AllocationPolicy::new(50, 10, 40, 0).unwrap();
        let projector = GoalFundProjector::new(500_000, 100_000, policy, 2_000_000);
        let _ = projector.project_timeline(months);
    }
}
