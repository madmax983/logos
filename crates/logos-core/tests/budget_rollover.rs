#[test]
fn rollover_conserves_value() {
    let end = logos_core::rollover_end_balance(100_000, 50_000, 120_000);
    assert_eq!(end, 30_000);
}

#[test]
fn rollover_handles_overspend() {
    let end = logos_core::rollover_end_balance(20_000, 10_000, 45_000);
    assert_eq!(end, -15_000);
}

#[test]
fn rollover_saturates_on_negative_overflow() {
    let end = logos_core::rollover_end_balance(i64::MIN, 0, 1);
    assert_eq!(end, i64::MIN);
}

#[test]
fn rollover_saturates_on_positive_overflow() {
    let end = logos_core::rollover_end_balance(i64::MAX, 1, -1);
    assert_eq!(end, i64::MAX);
}

#[test]
fn rollover_preserves_value_when_positive_overflow_is_compensated() {
    let end = logos_core::rollover_end_balance(i64::MAX, 1, 1);
    assert_eq!(end, i64::MAX);
}

#[test]
fn rollover_preserves_value_when_negative_overflow_is_compensated() {
    let end = logos_core::rollover_end_balance(i64::MIN, -1, -1);
    assert_eq!(end, i64::MIN);
}
