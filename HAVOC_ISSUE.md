Title: "👺 Havoc: `CashflowProjector` Panics on Arithmetic Overflow"

🧨 **The Trigger:**
Passing `i64::MAX` to `project_balances` causes an arithmetic overflow panic when simulating recurring cashflows. The `+=` operator in `*balance += posting.amount();` is unguarded.

📉 **The Stack Trace:**
```
attempt to add with overflow
```

🧪 **Reproduction:**
Run `cargo test -p logos-core project_balances_panics_on_overflow` to see the panics in `crates/logos-core/tests/havoc_proptest.rs`.

😈 **Comment:**
"You assumed people wouldn't project cashflows larger than RAM. You were wrong. A single massive salary input completely crashes the cashflow simulation."

---

Title: "👺 Havoc: `CategoryTrendAnalyzer` Panics on Arithmetic Overflow"

🧨 **The Trigger:**
Passing large positive transaction postings (e.g., `i64::MAX`) and analyzing them causes an arithmetic addition overflow panic. The `+=` operator in `*total += posting.amount();` is unguarded.

📉 **The Stack Trace:**
```
attempt to add with overflow
```

🧪 **Reproduction:**
Run `cargo test -p logos-core test_category_trends_panics_on_overflow --features nova` to see the panics in `crates/logos-core/tests/havoc_category_trends.rs`.

😈 **Comment:**
"You assumed category group totals wouldn't overflow `i64` capacity. You were wrong. A single massive expense completely crashes the trend analysis."
