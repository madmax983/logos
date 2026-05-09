#![allow(clippy::should_panic_without_expect)]

use logos_core::fire::{FireConfig, FireSimulator};
use logos_core::fire_ascent::FireAscentSimulator;
use logos_core::net_worth_projector::NetWorthProjector;
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_fire_ascent_panics_on_overflow(
        monthly_expenses in (i64::MAX / 20)..(i64::MAX / 10),
    ) {
        let mut fire_sim = FireSimulator::new(monthly_expenses);
        fire_sim.set_config(FireConfig { safe_withdrawal_rate_pct: 2 });
        let projector = NetWorthProjector::new(0, 0);
        let ascent_sim = FireAscentSimulator::new(fire_sim, projector, 60);
        let result = ascent_sim.ascend();
        assert!(!result.success);
    }
}
