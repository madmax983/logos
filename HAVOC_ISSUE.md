Title: "👺 Havoc: `CashflowProjector` Panics on Arithmetic Overflow"

🧨 **The Trigger:**
Passing `i64::MAX` to `project_balances` causes an arithmetic overflow panic when simulating recurring cashflows. The `+=` operator in `current_balances.entry(posting.account().as_str().to_owned()).or_insert(0) += posting.amount();` is unguarded.

📉 **The Stack Trace:**
```
thread 'project_balances_panics_on_overflow' panicked at src/experimental/cashflow_projector.rs:182:
attempt to add with overflow
```

🧪 **Reproduction:**
Run `cargo test -p logos-core` to see the panics in `crates/logos-core/tests/havoc_proptest.rs`.

😈 **Comment:**
"You assumed people wouldn't project cashflows larger than RAM. You were wrong. A single massive salary input completely crashes the cashflow simulation."
