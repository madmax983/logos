Title: "👺 Havoc: `SubscriptionFatigueAnalyzer` Panics on Arithmetic Overflow"

🧨 **The Trigger:**
Passing massively huge recurring subscription costs (e.g., `i64::MAX`) causes an arithmetic overflow when accumulating totals. The `+=` operator in `total_monthly += cost_result.monthly_cost_cents;` and `total_opportunity += cost_result.future_value_cents;` is unguarded.

📉 **The Stack Trace:**
```
thread 'subscription_fatigue_panics_on_overflow' panicked at library/core/src/ops/arith.rs:832:1:
attempt to add with overflow
```

🧪 **Reproduction:**
Run `cargo test -p logos-core havoc_subscription_fatigue --features nova`

😈 **Comment:**
"You assumed people wouldn't have subscriptions that cost more than global GDP. You were wrong. The analyzer crashes completely when the combined total overflows."
