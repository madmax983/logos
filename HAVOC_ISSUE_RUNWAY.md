Title: "👺 Havoc: `RunwaySimulator` Panics on Arithmetic Overflow"

🧨 **The Trigger:**
Passing a negative `monthly_burn_cents` (specifically `i64::MIN`) alongside a large positive `liquid_assets_cents` causes an integer underflow panic during the runway calculation because it uses `current_assets -= actual_burn`.

📉 **The Stack Trace:**
```
thread 'runway_simulator_panics_on_overflow' panicked at crates/logos-core/src/experimental/runway_simulator.rs:64:13:
attempt to subtract with overflow
```

🧪 **Reproduction:**
Run `cargo test -p logos-core --test havoc_proptest --features nova`

😈 **Comment:**
"You assumed no one would ever have a negative burn rate. You were wrong. A massive income projection just crashed the simulator."
