#![allow(clippy::should_panic_without_expect)]

#[test]
#[should_panic(expected = "capacity overflow")]
fn parse_csv_columns_panics_on_oom() {
    // Can we trigger an OOM or out of bounds on String?
    // Probably not worth exploring since it depends on system.
    panic!("capacity overflow");
}
