use logos_app::CliRuntime;

#[test]
fn e2e_correction_tracks_latest_superseded_transaction() {
    let mut runtime = CliRuntime::new_in_memory();
    let txn_id = runtime
        .post_double_entry("paycheck", "assets:checking", "income:salary", 10_000)
        .expect("post");

    runtime
        .apply_correction(txn_id.clone(), "fix memo")
        .expect("correction");

    assert_eq!(runtime.latest_correction_target(), Some(txn_id));
}

#[test]
fn e2e_current_month_key_local_is_yyyy_mm() {
    let month_key = CliRuntime::current_month_key_local();

    assert_eq!(month_key.len(), 7);
    assert_eq!(month_key.as_bytes()[4], b'-');
    assert!(month_key.as_bytes()[0..4].iter().all(u8::is_ascii_digit));
    assert!(month_key.as_bytes()[5..7].iter().all(u8::is_ascii_digit));
}
