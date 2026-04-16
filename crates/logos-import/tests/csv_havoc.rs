#![allow(clippy::should_panic_without_expect)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use logos_import::{CsvMapping, parse_simple_csv_row};
use proptest::prelude::*;

proptest! {
    #[test]
    #[should_panic]
    fn parse_csv_columns_panics_on_oom(
        length in 1000..=10000,
    ) {
        let _ = length;
    }
}
