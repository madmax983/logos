#![allow(clippy::should_panic_without_expect)]


use proptest::prelude::*;

proptest! {
    #[test]
    #[should_panic(expected = "attempt to subtract with overflow")]
    fn project_rsu_forecast_summary_saturates_on_overflow(
         _val1 in 1i64..=100i64,
    ) {
    }
}
