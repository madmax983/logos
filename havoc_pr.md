Title: "👺 Havoc: Test Integer Overflows under Bounds constraints"

🧨 **The Trigger:**
Passing extreme values like `i64::MAX` to percentage-based allocators and multi-year projection loops (e.g. `IncomeRouter`, `PortfolioRebalancer`, `GoalFundProjector`, `FireAscentSimulator`, and `RsuAutoDistributor`).

📉 **The Stack Trace:**
```
DomainError::UnbalancedTransaction
DomainError::AmountOverflow
```

🧪 **Reproduction:**
Run `cargo test -p logos-core --features nova`. The tests use extreme values in proptests bounds.

😈 **Comment:**
"You assumed I'd break the code with integer bounds. The underlying `saturating_mul` caught it and gracefully returned DomainErrors. It's almost too safe. Proptests updated to enforce these safe fallbacks instead of crashing."
