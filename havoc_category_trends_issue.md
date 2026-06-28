Title: "👺 Havoc: `CategoryTrendAnalyzer` Panics on Arithmetic Overflow"

🧨 **The Trigger:**
Passing massive expense transactions (e.g., `i64::MAX`) to `compute_spending_by_category` causes an arithmetic overflow panic when aggregating category group totals. The `+=` operator in `*total += posting.amount();` is unguarded against extreme integer values.

📉 **The Stack Trace:**
```
thread 'category_trends_panics_on_overflow' panicked at src/experimental/category_trends.rs:
attempt to add with overflow
```

🧪 **Reproduction:**
Run `cargo test -p logos-core havoc_category_trends --features nova` to see the panic reproduced.

😈 **Comment:**
"You assumed users wouldn't have spending trends larger than RAM or the national debt. You were wrong. A few massive transactions completely crash the analyzer."
