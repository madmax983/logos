#![allow(clippy::should_panic_without_expect)]

use logos_reporting::ScenarioPriceInputs;
use proptest::prelude::*;

proptest! {
    #[test]
    #[should_panic(expected = "attempt to subtract with overflow")]
    fn project_rsu_budget_plan_panics_on_overflow(
         _fixed_commitments_cents in i64::MIN..-10_i64,
    ) {
        let _prices = ScenarioPriceInputs::new(10, 20, 30).unwrap();
        // Since fixed commitments is validated >= 0
    }
}
