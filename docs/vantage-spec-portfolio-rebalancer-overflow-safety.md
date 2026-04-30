# 🔭 Vantage: Spec for Portfolio Rebalancer Overflow Safety

👤 **User Story:**
"As an investor rebalancing my portfolio, I want the system to safely handle extremely large total portfolio values without crashing, so that a massive account balance doesn't break the entire rebalancing calculation."

🤔 **So What?**
A financial tool that panics on unexpected inputs is unreliable and untrustworthy. When users explore edge cases (e.g., massive institutional portfolios or simulated extreme scenarios), the software should gracefully report an error or cap results, rather than suffering a hard crash that destroys the user session. Complexity is a cost, but stability is a requirement.

🎯 **Metric Definition:**
Success = 0 panics during portfolio rebalancing, even when providing inputs like `i64::MAX`. The calculation must complete and return a capped value, explicit error, or safely processed amounts.

🔍 **Gap Analysis:**
Currently, the `rebalance` function in `PortfolioRebalancer` uses unguarded operators for calculating totals and percentage allocations (e.g., `total_value += balance` and `let allocated = (total_value * i64::from(target.percentage)) / 100;`). Passing a massive balance like `i64::MAX` causes a multiplication arithmetic overflow panic. This assumes the intermediate math will never exceed the 64-bit integer limit. Standard financial simulation engines handle extreme edge cases safely by using saturating arithmetic or bounds checking.

✅ **Acceptance Criteria:**
- The rebalancing engine must not panic when computing totals or percentage allocations for extremely large portfolio balances.
- The allocation logic must be updated to handle potential arithmetic overflows safely (e.g., using saturating math or returning an explicit error).
- All existing tests, including chaos tests that trigger this overflow, must pass successfully.

🚫 **Out of Scope:**
- Rewriting the entire rebalancing engine to use arbitrary-precision numbers.
