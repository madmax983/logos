Title: "👺 Havoc: `RunwaySimulator` Panics on Arithmetic Overflow"

🧨 **The Trigger:**
Passing large negative `monthly_burn_cents` values (like `i64::MIN`) to `RunwaySimulator::new` and calling `calculate_runway` causes an arithmetic subtraction overflow panic. When `actual_burn` resolves to `i64::MIN`, the `-=` operator in `current_assets -= actual_burn;` attempts to subtract `-MAX`, causing an unguarded overflow.

📉 **The Stack Trace:**
```
thread 'havoc_runway_simulator_panics_on_overflow' panicked at crates/logos-core/src/experimental/runway_simulator.rs:71:
attempt to subtract with overflow
```

🧪 **Reproduction:**
Run `cargo test -p logos-core --test havoc_runway_simulator --features nova` to see the panics.

😈 **Comment:**
"You assumed no one could ever pass negative burn rates. You were wrong. A single malicious calculation completely crashes the simulation."
