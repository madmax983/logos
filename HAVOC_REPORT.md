👺 Havoc: Proving Integer Fragility in Core Planning Modules

🧨 **The Trigger:** Inputting an upcoming vest with `avg_close_price_cents` approaching `i64::MAX / 100` and `units` > 100_000 causes a buffer overflow when summing the values in `safe_net_worth_cents()`.

📉 **The Stack Trace:**
```
thread 'safe_net_worth_cents_panics_on_overflow' panicked at core/src/num/mod.rs:1145:5:
attempt to add with overflow
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

🧪 **Reproduction:** Run `cargo test --test havoc_proptest safe_net_worth_cents_panics_on_overflow`.

😈 **Comment:** You assumed the user wouldn't vest millions of dollars. You were wrong.
