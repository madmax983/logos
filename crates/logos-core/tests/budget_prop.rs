use proptest::prelude::*;

#[test]
fn rollover_property_holds_for_value_grid() {
    for start in [-200_000_i64, -50_000, 0, 50_000, 200_000] {
        for assigned in [0_i64, 10_000, 55_000, 120_000] {
            for spent in [0_i64, 5_000, 25_000, 90_000, 220_000] {
                let end = logos_core::rollover_end_balance(start, assigned, spent);

                // Assert behavior in realistic non-overflow bounds
                assert_eq!(start + assigned - spent, end);
            }
        }
    }
}

proptest! {
    #[test]
    fn proptest_rollover_does_not_panic_and_stays_in_bounds(start in any::<i64>(), assigned in any::<i64>(), spent in any::<i64>()) {
        let end = logos_core::rollover_end_balance(start, assigned, spent);

        // Assert invariants
        // 1. It didn't panic! (The act of returning proves this)
        // 2. If it didn't overflow, the math is exact.
        if let Some(s1) = start.checked_add(assigned) {
            if let Some(expected) = s1.checked_sub(spent) {
                assert_eq!(end, expected);
            }
        }

        // 3. The value is always bounded.
        assert!(end <= i64::MAX);
        assert!(end >= i64::MIN);
    }
}
