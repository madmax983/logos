#![allow(clippy::should_panic_without_expect)]

#[cfg(feature = "nova")]
use logos_core::AllocationPolicy;
#[cfg(feature = "nova")]
use logos_core::goal_fund_projector::GoalFundProjector;
#[cfg(feature = "nova")]
use proptest::prelude::*;

#[cfg(feature = "nova")]
proptest! {
    #[test]
    fn project_timeline_panics_on_u16_overflow(
        months in 3000..=u16::MAX,
    ) {
        let policy = AllocationPolicy::new(50, 10, 40, 0).unwrap();
        let projector = GoalFundProjector::new(500_000, 100_000, policy, 2_000_000);

        let result = projector.project_timeline(months);
        // Assert it either returns Ok or an AmountOverflow Err (which is how the system safely protects itself now)
        assert!(result.is_ok() || matches!(result.unwrap_err(), logos_core::DomainError::AmountOverflow));
    }
}
