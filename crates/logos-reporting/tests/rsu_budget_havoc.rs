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
}
