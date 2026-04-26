Title: "👺 Havoc: `IncomeRouter` Panics on Arithmetic Overflow"

🧨 **The Trigger:**
Passing `i64::MAX` to `route_income` causes an arithmetic multiplication overflow when calculating the percentage allocation. The `amount_cents * i64::from(rule.percentage)` expression is unguarded.

📉 **The Stack Trace:**
```
thread 'income_router_panics_on_overflow' panicked at crates/logos-core/src/experimental/income_router.rs:70:29:
attempt to multiply with overflow
```

🧪 **Reproduction:**
Run `cargo test -p logos-core --test income_router_havoc --features nova`.

😈 **Comment:**
"You assumed people wouldn't route paychecks larger than RAM. You were wrong. A single massive salary input completely crashes the income routing."
