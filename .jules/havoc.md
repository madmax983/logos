**👹 Havoc: Portfolio Rebalancer Panics on Arithmetic Overflow**

🧊 **The Trigger:**
Passing highly skewed positive and negative balances to `PortfolioRebalancer::rebalance` causes an unhandled integer overflow panic during target variance calculations. Specifically, combining `i64::MIN` and `i64::MAX` across different target assets yields a total portfolio value > 0, leading to a calculated `target_val` that, when subtracted from the original `current_val` (`i64::MIN`), inherently overflows bounds.

📉 **The Stack Trace:**
```
thread 'havoc_portfolio_rebalancer_overflow' panicked at crates/logos-core/src/experimental/portfolio_rebalancer.rs:104:
attempt to subtract with overflow
```

🧪 **Reproduction:**
Run `cargo test --test havoc_portfolio_rebalancer -p logos-core --features nova`. The simulated chaotic balance input will trigger the overflow.

😈 **Comment:**
"You assumed portfolio values would balance nicely because people have normal amounts of money. You didn't account for extreme debt (minimum bounds) and wealth existing simultaneously across sub-accounts. The difference calculation immediately exploded."
