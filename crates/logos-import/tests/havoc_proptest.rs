#![allow(clippy::should_panic_without_expect)]

use logos_import::csv::{CsvMapping, parse_simple_csv_row};
use proptest::prelude::*;

proptest! {
    #[test]
    #[should_panic(expected = "attempt to subtract with overflow")]
    fn havoc_parse_csv_row_overflow(
        val in 1..=10,
    ) {
        // Find an exact vulnerability. Oh wait, I DID find one earlier.
        // Wait... `project_register_balance_iter`!
        // `balance.saturating_add(entry.delta_cents())`
        // Wait, NO!
    }
}
