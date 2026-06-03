👺 Havoc: `civil_from_days` Date Overflow Panic

🧨 **The Trigger:**
When passing extreme positive inputs near the bounds of `i64::MAX` to the `civil_from_days` logic inside `logos-tui`, the initial assignment `let z = days_since_unix_epoch + 719_468;` attempts to add the unix offset to a highly positive value without bounds checking. Due to the lack of checked or saturating addition, this directly causes an arithmetic overflow panic in debug mode, and silently wraps memory in release mode. While `civil_from_days` currently receives bounded inputs scaled down from Unix timestamps in practice, as a utility function parsing any raw day count, it is highly fragile to boundary inputs.

📉 **The Stack Trace:**
```
thread 'app::havoc_tests::test_civil_from_days_panic' panicked at crates/logos-tui/src/app.rs:1550:13:
attempt to add with overflow
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

🧪 **Reproduction:**
Run the following test inside `crates/logos-tui/src/app.rs`:
```rust
#[cfg(test)]
mod havoc_tests {
    use super::*;

    #[test]
    #[should_panic]
    fn test_civil_from_days_panic() {
        let _ = civil_from_days(i64::MAX);
    }
}
```
Run `cargo test -p logos-tui` and witness the test pass successfully due to the expected arithmetic panic.

😈 **Comment:**
You assumed time would never reach the end of the universe plus 719,468 days. You were wrong.
