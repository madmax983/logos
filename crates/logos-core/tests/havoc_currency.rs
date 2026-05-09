// Havoc mode: Property test targeting integer boundaries
// The currency formatting function takes an i64, representing cents.
// We expect it to break on the exact edge case of i64::MIN due to
// two's complement overflow when calculating the absolute value.

use logos_core::format::currency;
use proptest::prelude::*;

// We force `proptest` to test `i64::MIN` by explicitly including it
// as part of the domain using a strategy that favors boundaries, or by
// doing a dedicated test since `any::<i64>()` has a low chance of hitting
// exact boundaries. The chaos instructions require failing tests.

proptest! {
    #[test]
    #[should_panic(expected = "attempt to negate with overflow")]
    // Havoc: We *expect* a panic when a negative overflow is hit at formatting time.
    fn test_currency_proptest_boundary(cents in prop::sample::select(vec![std::i64::MIN])) {
        let _ = currency(cents);
    }
}
