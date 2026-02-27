use logos_cli::runtime::CliRuntime;

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
