#[test]
fn rollover_property_holds_for_value_grid() {
    for start in [-200_000_i64, -50_000, 0, 50_000, 200_000] {
        for assigned in [0_i64, 10_000, 55_000, 120_000] {
            for spent in [0_i64, 5_000, 25_000, 90_000, 220_000] {
                let end = logos_core::rollover_end_balance(start, assigned, spent);
                assert_eq!(start + assigned - spent, end);
            }
        }
    }
}
