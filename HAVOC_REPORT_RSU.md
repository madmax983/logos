Title: "👺 Havoc: `project_rsu_forecast_summary` Panics on Arithmetic Overflow"

🧨 **The Trigger:**
Passing `i64::MAX` inside the `projected_events_cents` slice to `project_rsu_forecast_summary` along with another positive value causes an arithmetic buffer overflow when summing the events.

📉 **The Stack Trace:**
```
thread 'project_rsu_forecast_summary_panics_on_overflow' panicked at crates/logos-reporting/src/rsu_forecast.rs:21:56:
attempt to add with overflow
```

🧪 **Reproduction:**
Run `cargo test -p logos-reporting --test rsu_forecast_havoc`

😈 **Comment:**
"You assumed RSU events would never sum to more than the 64-bit integer limit. You were wrong. A billionaire forecasting their RSUs just crashed the reporting engine."
