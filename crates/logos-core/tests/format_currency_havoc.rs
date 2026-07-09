#![allow(clippy::should_panic_without_expect)]

use logos_core::format::currency;
use proptest::prelude::*;

proptest! {
    #[test]
    #[should_panic(expected = "attempt to negate with overflow")]
    fn havoc_currency_format_panics_on_min_i64(
        _dummy in 0..1_i32,
    ) {
        let _ = currency(i64::MIN);
    }
}
