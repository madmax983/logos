#![allow(clippy::should_panic_without_expect)]

use logos_reporting::register::{RegisterEntry, project_register_balance_iter};
use logos_reporting::rsu_forecast::project_rsu_forecast_summary;
use proptest::prelude::*;

proptest! {
    #[test]
    fn project_register_balance_iter_saturates_on_overflow(
        opening_balance in i64::MAX..=i64::MAX,
        delta1 in 1i64..=100i64,
    ) {
        let entries = [
            RegisterEntry::new(delta1),
        ];

        let result = project_register_balance_iter(opening_balance, entries);
        assert_eq!(result, i64::MAX);
    }

    #[test]
    #[should_panic(expected = "attempt to add with overflow")]
    fn project_rsu_forecast_summary_panics_on_overflow(
        val1 in i64::MAX..=i64::MAX,
        val2 in 1i64..=1i64,
    ) {
        let _result = project_rsu_forecast_summary(&[val1, val2]);
    }
}
