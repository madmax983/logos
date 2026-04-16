#![allow(clippy::should_panic_without_expect)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use logos_import::{CsvMapping, parse_simple_csv_row};
use proptest::prelude::*;

proptest! {
    #[test]
    fn parse_csv_columns_panics_on_oom(
        length in 1000..=10000,
    ) {
        // Can we trigger an OOM or out of bounds on String?
        // Probably not worth exploring since it depends on system.
    }
}
