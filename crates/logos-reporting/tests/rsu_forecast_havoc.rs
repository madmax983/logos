#![allow(clippy::should_panic_without_expect)]

use logos_reporting::rsu_forecast::project_rsu_forecast_summary;
use proptest::prelude::*;

proptest! {
    #[test]
    #[should_panic(expected = "attempt to add with overflow")]
    fn project_rsu_forecast_summary_panics_on_overflow(
        val1 in i64::MAX..=i64::MAX,
        val2 in 1i64..=1i64,
    ) {
        let events = [val1, val2];
        let _ = project_rsu_forecast_summary(&events);
    }
}
