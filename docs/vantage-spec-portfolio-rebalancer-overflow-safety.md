# 🔭 Vantage: Spec for Portfolio Rebalancer Overflow Safety

## 👤 User Story:
"As an investor rebalancing a massive portfolio, I want the system to safely handle extremely large balances without crashing, so that edge-case high net worth scenarios don't break the tool."

## 🤔 So What?
A financial tool that panics on unexpected inputs is unreliable and untrustworthy. When users explore edge cases (e.g., massive hypothetical portfolios for stress testing), the software should gracefully report an error or cap results, rather than suffering a hard crash that destroys the entire user session.

## 🎯 Metric Definition:
Success = 0 panics during portfolio rebalancing calculations, even when providing inputs like `i64::MAX`. The calculation must complete and return a capped value or explicit error.

## 🔎 Gap Analysis:
Currently, the `rebalance` calculation in `PortfolioRebalancer` uses an unguarded `*` operator for calculating percentage allocations against the total portfolio value. This assumes the intermediate multiplication will never exceed the 64-bit integer limit. Standard financial simulation engines handle edge cases safely by checking for overflow.

## ✅ Acceptance Criteria:
- The `rebalance` logic must not panic when allocating percentages of extremely large portfolio values.
- The allocation logic must be updated to handle potential arithmetic overflows safely.
- All existing tests, including chaos/proptests that trigger this overflow, must pass successfully after the fix.

## 🚫 Out of Scope:
- Rewriting the entire rebalancing engine to use arbitrary-precision numbers.
