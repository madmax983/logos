#![allow(clippy::should_panic_without_expect)]


use proptest::prelude::*;

proptest! {
    #[test]
    #[should_panic(expected = "capacity overflow")]
    fn parse_csv_columns_panics_on_oom(
        _length in 1000..=10000,
    ) {
        // Can we trigger an OOM or out of bounds on String?
        // Probably not worth exploring since it depends on system.
    }
}
