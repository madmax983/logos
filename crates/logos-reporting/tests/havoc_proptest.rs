#![allow(clippy::should_panic_without_expect)]

use logos_reporting::register::{project_register_balance_iter, RegisterEntry};
use proptest::prelude::*;

proptest! {
    #[test]
    #[should_panic(expected = "attempt to add with overflow")]
    fn project_register_balance_iter_panics_on_overflow(
        opening_balance in any::<i64>(),
        delta1 in any::<i64>(),
        delta2 in any::<i64>(),
    ) {
        let entries = [
            RegisterEntry::new(delta1),
            RegisterEntry::new(delta2),
        ];

        let _ = project_register_balance_iter(opening_balance, entries);
    }
}
