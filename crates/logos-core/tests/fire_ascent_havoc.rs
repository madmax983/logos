use logos_core::experimental::fire_ascent::FireAscentSimulator;
use logos_core::planning::fire::{FireConfig, FireSimulator};
use logos_core::planning::net_worth_projector::NetWorthProjector;

#[test]
fn test_impossible_ascent() {
    let mut fire_sim = FireSimulator::new(400_000);
    fire_sim.set_config(FireConfig {
        safe_withdrawal_rate_pct: 0,
    });

    let projector = NetWorthProjector::new(80_000_000, 1_000_000);

    let ascent_sim = FireAscentSimulator::new(fire_sim, projector, 60);
    let result = ascent_sim.ascend();

    assert!(result.impossible);
    assert!(!result.success);
    assert_eq!(result.summit_cents, i64::MAX);
}
