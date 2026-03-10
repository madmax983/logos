#[test]
fn logos_fetch_crate_is_available() {
    assert!(logos_fetch::crate_ready());
}
