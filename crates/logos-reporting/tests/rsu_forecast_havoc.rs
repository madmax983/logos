#![allow(clippy::should_panic_without_expect)]

use logos_reporting::rsu_forecast::project_rsu_forecast_summary;
use proptest::prelude::*;

proptest! {
    #[test]
    #[should_panic]
    fn project_rsu_forecast_summary_panics_on_overflow(
        val1 in 1i64..=100i64,
    ) {
        let events = [i64::MAX, val1];
        let _ = project_rsu_forecast_summary(&events);
    }
}
