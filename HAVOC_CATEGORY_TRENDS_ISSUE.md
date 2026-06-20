Title: "👺 Havoc: `CategoryTrendAnalyzer` Panics on Arithmetic Overflow"

🧨 **The Trigger:**
Passing large transaction amounts (e.g., `i64::MAX / 2 + 1`) to `compute_spending_by_category` causes an arithmetic overflow panic when aggregating category totals. The `+=` operator in `*total += posting.amount();` is unguarded.

📉 **The Stack Trace:**
```
attempt to add with overflow
```

🧪 **Reproduction:**
Run `cargo test -p logos-core --features nova --test category_trends_havoc`.

😈 **Comment:**
"You assumed total category spending would never exceed RAM. You were wrong. A few massive transactions completely crash the trend analyzer."
