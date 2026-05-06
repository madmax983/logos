#![allow(clippy::should_panic_without_expect)]

use logos_reporting::{RsuBudgetPlanInput, ScenarioPriceInputs, project_rsu_budget_plan};
use proptest::prelude::*;

proptest! {
    #[test]
    fn project_rsu_budget_plan_saturates_on_overflow(
        fixed_commitments_cents in 0_i64..i64::MAX,
    ) {
        let prices = ScenarioPriceInputs::new(10, 20, 30).unwrap();
        let input = RsuBudgetPlanInput::new(100, 0, prices, fixed_commitments_cents, 10, 20).unwrap();
        let plan = project_rsu_budget_plan("2024-05", &input).unwrap();
        assert!(plan.conservative_budget_cents() >= 0);
    }

    #[test]
    fn project_rsu_budget_plan_saturates_prices_on_overflow(
        bear in 1_i64..i64::MAX / 2,
        base_offset in 0_i64..i64::MAX / 4,
        bull_offset in 0_i64..i64::MAX / 4,
        units in 1_u32..u32::MAX,
    ) {
        let base = bear.saturating_add(base_offset);
        let bull = base.saturating_add(bull_offset);
        if let Ok(prices) = ScenarioPriceInputs::new(bear, base, bull) {
            if let Ok(input) = RsuBudgetPlanInput::new(units, 0, prices, 5000, 10, 20) {
                let plan = project_rsu_budget_plan("2024-05", &input).unwrap();
                let bear_scenario = plan.bear().unwrap();
                let base_scenario = plan.base().unwrap();
                let bull_scenario = plan.bull().unwrap();

                assert!(bear_scenario.surplus_cents() >= 0);
                assert!(base_scenario.surplus_cents() >= 0);
                assert!(bull_scenario.surplus_cents() >= 0);
                assert!(base_scenario.investing_sweep_cents() >= 0);
                assert!(bull_scenario.investing_sweep_cents() >= 0);
            }
        }
    }
}
