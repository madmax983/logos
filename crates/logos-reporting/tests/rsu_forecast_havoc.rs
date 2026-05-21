#![allow(clippy::should_panic_without_expect)]
#![allow(clippy::tuple_array_conversions)]

use logos_reporting::project_rsu_forecast_summary;
use proptest::prelude::*;

proptest! {
    #[test]
    fn project_rsu_forecast_summary_saturates_on_overflow(
        val1 in i64::MAX - 10..=i64::MAX,
        val2 in i64::MAX - 10..=i64::MAX,
    ) {
        let array = [val1, val2];
        let summary = project_rsu_forecast_summary(&array);
        assert_eq!(summary.projected_total_cents(), i64::MAX);
    }

    #[test]
    fn project_rsu_forecast_summary_saturates_on_underflow(
        val1 in i64::MIN..=i64::MIN + 10,
        val2 in i64::MIN..=i64::MIN + 10,
    ) {
        let array = [val1, val2];
        let summary = project_rsu_forecast_summary(&array);
        assert_eq!(summary.projected_total_cents(), i64::MIN);
    }
}
