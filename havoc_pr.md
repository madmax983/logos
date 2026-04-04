# 👺 Havoc: forecast_value_cents panics on large gross values

🧨 **The Trigger:**
Passing a large `avg_close_price_cents` (e.g. `i64::MAX / 50`) and `units = 1` to `forecast_value_cents` computes a `gross_value` that, when multiplied by the `retained_pct` (e.g. 75), overflows the integer bounds and panics.

📉 **The Stack Trace:**
```
thread 'forecast_value_cents_panics_on_large_gross_value' panicked at crates/logos-core/src/domain/rsu.rs:232:5:
attempt to multiply with overflow
```

🧪 **Reproduction:**
Run `cargo test -p logos-core test_forecast_value_cents_panics_on_large_gross_value`.

😈 **Comment:**
You assumed stock prices would never be astronomical. You were wrong.
