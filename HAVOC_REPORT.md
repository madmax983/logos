Title: "👺 Havoc: `project_register_balance_iter` Panics on Arithmetic Overflow"

🧨 **The Trigger:**
Passing `i64::MAX` to `project_register_balance_iter` along with a positive `delta_cents` causes an arithmetic buffer overflow when summing the register entries.

📉 **The Stack Trace:**
```
thread 'project_register_balance_iter_panics_on_overflow' panicked at crates/logos-reporting/src/register.rs:77:13:
attempt to add with overflow
```

🧪 **Reproduction:**
Run `cargo test -p logos-reporting --test havoc_proptest`

😈 **Comment:**
"You assumed register entries would never sum to more than the 64-bit integer limit. You were wrong. A billionaire entering their portfolio balance just crashed the reporting engine."
