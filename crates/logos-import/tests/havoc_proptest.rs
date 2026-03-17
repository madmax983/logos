#![allow(clippy::should_panic_without_expect)]

use logos_import::{CsvMapping, parse_simple_csv_row};
use proptest::prelude::*;

proptest! {
    #[test]
    fn parse_simple_csv_row_panics_on_overflow(
        col1 in any::<String>(),
        col2 in any::<String>(),
        col3 in any::<String>(),
        col4 in any::<String>(),
        col5 in any::<String>(),
        col6 in any::<String>(),
    ) {
        let mapping = CsvMapping {
            timestamp_idx: usize::MAX,
            amount_idx: usize::MAX,
            memo_idx: usize::MAX,
            account_idx: usize::MAX,
            category_idx: usize::MAX,
            source_id: "test".to_string(),
        };
        let row = format!("{col1},{col2},{col3},{col4},{col5},{col6}");
        let result = parse_simple_csv_row(&row, &mapping);
        assert!(result.is_err());
    }
}
