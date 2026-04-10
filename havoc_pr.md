Title: "👺 Havoc: `FireAscentSimulator` Panics on Arithmetic Overflow"

🧨 **The Trigger:**
Setting an extremely large `monthly_expenses` (e.g., `i64::MAX / 20`) combined with a conservative `safe_withdrawal_rate_pct` (e.g., `2`) generates a `fire_number` that, when multiplied by 3 during the `camp3` calculation `(fire_number * 3) / 4`, causes an unhandled integer overflow and panics the simulation.

📉 **The Stack Trace:**
```
thread 'test_fire_ascent_panics_on_overflow' panicked at crates/logos-core/src/experimental/fire_ascent.rs:77:24:
attempt to multiply with overflow
```

🧪 **Reproduction:**
Run `cargo test -p logos-core --test fire_ascent_havoc`

😈 **Comment:**
"You assumed no one would ever have a FIRE number large enough to break 64-bit multiplication when charting their ascent. You were wrong."
