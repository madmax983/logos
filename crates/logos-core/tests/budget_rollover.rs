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
