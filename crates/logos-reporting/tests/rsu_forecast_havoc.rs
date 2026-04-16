#![allow(clippy::should_panic_without_expect)]
#![allow(clippy::tuple_array_conversions)]
#![allow(clippy::useless_conversion)]

use logos_reporting::project_rsu_forecast_summary;
use proptest::prelude::*;

proptest! {
    #[test]
    fn project_rsu_forecast_summary_saturates_on_overflow(
        val1 in i64::MAX - 10..=i64::MAX,
        val2 in i64::MAX - 10..=i64::MAX,
    ) {
        let summary = project_rsu_forecast_summary(&[val1.into(), val2.into()]);
        assert_eq!(summary.projected_total_cents(), i64::MAX);
    }
}
