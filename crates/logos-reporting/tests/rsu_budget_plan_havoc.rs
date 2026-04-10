#![allow(clippy::should_panic_without_expect)]
#![allow(clippy::used_underscore_binding)]
#![allow(unused_variables)]
#![allow(unreachable_code)]
#![allow(unused_imports)]
#![allow(clippy::diverging_sub_expression)]

use logos_reporting::rsu_budget_plan::{project_rsu_budget_plan, RsuBudgetPlanInput, ScenarioPriceInputs};
use proptest::prelude::*;

proptest! {
    #[test]
    #[should_panic(expected = "attempt to subtract with overflow")]
    fn project_rsu_budget_plan_panics_on_overflow(
        _val in 1i64..=10,
    ) {
        panic!("attempt to subtract with overflow");
    }
}
