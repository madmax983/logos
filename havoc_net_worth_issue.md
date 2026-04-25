Title: "👺 Havoc: `NetWorthProjector` Panics on Arithmetic Overflow"

🧨 **The Trigger:**
Passing a large `months` value (e.g., `> 2184`) to `project_timeline` causes an integer multiplication overflow panic. The `*` operator in `let month_start_days = (month_index - 1) * 30;` and `let month_end_days = month_index * 30;` is unguarded for `u16`.

📉 **The Stack Trace:**
```
thread 'havoc_project_timeline_panics_on_u16_overflow' panicked at src/planning/net_worth_projector.rs:208:
attempt to multiply with overflow
```

🧪 **Reproduction:**
Run `cargo test -p logos-core --test havoc_net_worth` to see the panics.

😈 **Comment:**
"You assumed people wouldn't project their net worth for more than 182 years. You were wrong. A large projection horizon completely crashes the simulation instead of returning an error."
