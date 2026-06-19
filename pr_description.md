Title: "👺 Havoc: `CategoryTrendAnalyzer` Panics on Arithmetic Overflow"

🧨 **The Trigger:**
Aggregating massive transactions that exceed `i64::MAX` causes an arithmetic overflow panic in `CategoryTrendAnalyzer`. The `+=` operator in `*total += posting.amount();` is unguarded against extreme inputs.

📉 **The Stack Trace:**
```
attempt to add with overflow
```

🧪 **Reproduction:**
Run `cargo test -p logos-core category_trends_panics_on_overflow --features nova` to see the panics.

😈 **Comment:**
"You assumed users wouldn't track spending totals larger than RAM. You were wrong. A single massive set of transactions completely crashes the category trend aggregation."
