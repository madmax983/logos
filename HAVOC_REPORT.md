Title: "👺 Havoc: `project_rsu_forecast_summary` Panics on Arithmetic Overflow"

🧨 **The Trigger:**
Passing `i64::MAX` alongside a positive integer to `project_rsu_forecast_summary` causes an arithmetic buffer overflow when summing the projected events due to an unguarded `.sum::<i64>()` call.

📉 **The Stack Trace:**
```
thread 'project_rsu_forecast_summary_panics_on_overflow' panicked at library/core/src/iter/traits/accum.rs:204:1:
attempt to add with overflow
```

🧪 **Reproduction:**
Run `cargo test -p logos-reporting --test rsu_forecast_havoc`

😈 **Comment:**
"You assumed the projected events would never sum to more than the 64-bit integer limit. You were wrong. A massive RSU vest just crashed the reporting engine."