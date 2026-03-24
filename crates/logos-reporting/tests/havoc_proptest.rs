#![allow(clippy::should_panic_without_expect)]

use logos_reporting::register::{RegisterEntry, project_register_balance_iter};
use proptest::prelude::*;

proptest! {
    #[test]
    #[should_panic(expected = "attempt to add with overflow")]
    fn project_register_balance_iter_panics_on_overflow(
        opening_balance in i64::MAX..=i64::MAX,
        delta1 in 1i64..=100i64,
    ) {
        let entries = [
            RegisterEntry::new(delta1),
        ];

        let _ = project_register_balance_iter(opening_balance, entries);
    }
}
