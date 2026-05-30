Title: "👺 Havoc: `civil_from_days` Panics on Arithmetic Overflow"

🧨 **The Trigger:**
Passing a large timestamp value (`i64::MAX`) to `civil_from_days` inside the TUI app logic causes an arithmetic overflow panic. The `+` operator in `let z = days_since_unix_epoch + 719_468;` is unguarded and panics when given an extreme future date.

📉 **The Stack Trace:**
```
thread 'app::havoc_tests::havoc_date_formatting_panics_on_overflow' panicked at crates/logos-tui/src/app.rs:1552:13:
attempt to add with overflow
```

🧪 **Reproduction:**
Run `cargo test -p logos-tui havoc_date_formatting_panics_on_overflow`.

😈 **Comment:**
"You assumed the application would never handle UI transactions or calculations from the deep future. You were wrong. An extreme timestamp completely crashes the UI component when it attempts to format the date."
