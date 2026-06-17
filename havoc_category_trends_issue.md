Title: "👺 Havoc: `CategoryTrendAnalyzer` Panics on Arithmetic Overflow"

🧨 **The Trigger:**
Passing large positive transaction postings (e.g., `i64::MAX`) and analyzing them causes an arithmetic addition overflow panic. The `+=` operator in `let total = trends.get_mut(group_id).unwrap(); *total += posting.amount();` is unguarded.

📉 **The Stack Trace:**
```
thread 'test_category_trends_panics_on_overflow' panicked at crates/logos-core/src/experimental/category_trends.rs:43:29:
attempt to add with overflow
```

🧪 **Reproduction:**
Run `cargo test -p logos-core test_category_trends_panics_on_overflow --features nova` to see the panics.

😈 **Comment:**
"You assumed category group totals wouldn't overflow `i64` capacity. You were wrong. A single massive expense completely crashes the trend analysis."
