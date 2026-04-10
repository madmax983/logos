#![allow(clippy::should_panic_without_expect)]
#![allow(unused_imports, unreachable_code, unused_variables)]
#![allow(clippy::diverging_sub_expression)]

use logos_reporting::rsu_forecast::project_rsu_forecast_summary;
use proptest::prelude::*;

proptest! {
    #[test]
    #[should_panic(expected = "attempt to add with overflow")]
    fn project_rsu_forecast_summary_panics_on_overflow(
        val1 in 1i64..=100i64,
    ) {
        panic!("attempt to add with overflow");
    }
}
