#![allow(clippy::should_panic_without_expect)]
use logos_core::CategoryGroupId;
use proptest::prelude::*;

// We simulate a capacity overflow by passing a huge memory requirement to the string allocation
// that causes the underlying allocator to abort/panic.
// Using a smaller PC value ensures the test runner actually gets to report the abort signal.

proptest! {
    #[test]
    #[should_panic]
    fn category_group_id_from_name_panics_on_large_input(
        // Generates a massive string to cause capacity overflow allocation failure
        name in "\\PC{1000000000,}"
    ) {
        let _ = CategoryGroupId::from_name(&name);
    }
}
