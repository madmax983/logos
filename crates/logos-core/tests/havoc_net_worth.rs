use proptest::prelude::*;
use logos_core::planning::net_worth_projector::NetWorthProjector;

proptest! {
    #[test]
    fn test_project_timeline_does_not_panic(
        months in 0..=u16::MAX,
        initial_net_worth in any::<i64>(),
        monthly_savings in any::<i64>(),
    ) {
        let projector = NetWorthProjector::new(initial_net_worth, monthly_savings);
        let _ = projector.project_timeline(months);
    }
}
