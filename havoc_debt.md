Title: "👺 Havoc: DebtOptimizer Panics on Arithmetic Overflow"

🧨 **The Trigger:**
Passing `i64::MAX` to the `balance_cents` field of a `Debt` struct in `DebtOptimizer` and calling `simulate` causes an arithmetic overflow panic. The calculation `(debt.balance_cents * i64::from(debt.interest_rate_pct)) / 100 / 12` performs unprotected multiplication against massive numbers, which will exceed `i64::MAX`.

📉 **The Stack Trace:**
```
thread 'debt_optimizer_simulate_panics_on_overflow' panicked at src/experimental/debt_optimizer.rs:82:
attempt to multiply with overflow
```

🧪 **Reproduction:**
Run `cargo test -p logos-core` to see the panics in `crates/logos-core/tests/havoc_proptest.rs`.

😈 **Comment:**
"You assumed people wouldn't simulate debts larger than the global GDP. You were wrong. A single massive debt input completely crashes the debt payoff simulation. Check your bounds."
