#![allow(clippy::should_panic_without_expect)]

use logos_import::{CsvMapping, parse_simple_csv_row};
use proptest::prelude::*;

proptest! {
    #[test]
    fn parse_csv_columns_panics_on_oom(
        length in 1000..=10000,
    ) {
        let mapping = CsvMapping::default();
        let row = format!("{}, {}", "A".repeat(length as usize), "B".repeat(length as usize));

        let _ = std::panic::catch_unwind(|| {
            let _ = parse_simple_csv_row(&row, &mapping);
        });
    }
}
