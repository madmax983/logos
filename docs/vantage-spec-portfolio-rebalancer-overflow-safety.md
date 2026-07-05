# 🔭 Vantage: Spec for Portfolio Rebalancer Overflow Safety

👤 **User Story:**
"As a user rebalancing my investments, I want the system to safely handle extremely large portfolio balances without crashing, so that edge-case scenarios don't break the rebalancing calculation."

🤔 **So What? (Business Problem):**
Financial tools that panic on unexpected or extremely large inputs are unreliable. When users explore edge cases (e.g., massive portfolio values), the software should gracefully report an error or cap results rather than suffering a hard crash.

🎯 **Metric Definition:**
Success = 0 panics during portfolio rebalancing, even when providing inputs like `i64::MAX`. The calculation must complete and return a capped value or explicit error.

🔎 **Gap Analysis:**
Currently, `PortfolioRebalancer::rebalance` uses an unguarded `*` operator for calculating percentage allocations in the form `(total_value * i64::from(target.percentage)) / 100`. This assumes the intermediate multiplication will never exceed the 64-bit integer limit. Standard financial engines handle these edge cases safely by checking for overflow.

✅ **Acceptance Criteria:**
- The `rebalance` function must not panic when allocating extremely large portfolio values.
- The allocation logic must be updated to handle potential arithmetic overflows safely.
- All existing tests, including chaos tests like `portfolio_rebalancer_panics_on_overflow`, must pass successfully after the fix.

🚫 **Out of Scope:**
- Rewriting the entire rebalancing engine to use arbitrary-precision numbers.
