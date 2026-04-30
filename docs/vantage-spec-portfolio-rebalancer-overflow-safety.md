# 🔭 Vantage: Spec for Portfolio Rebalancer Overflow Safety

👤 **User Story:**
"As an investor utilizing the portfolio rebalancer, I want the system to safely handle extremely large portfolio balances without crashing, so that a massive total value doesn't break the rebalancing calculation."

🤔 **So What?**
A financial tool that panics on unexpected inputs is unreliable and untrustworthy. When users provide extreme edge case values (e.g., massive balances), the software should gracefully report an error or cap results, rather than suffering a hard crash. Resilient software builds trust; fragile software creates frustration.

🎯 **Metric Definition:**
Success = 0 panics during portfolio rebalancing, even when providing inputs like `i64::MAX`. The calculation must complete and return an explicit error or handle the overflow safely.

🔎 **Gap Analysis:**
Currently, the `rebalance` function uses an unguarded `*` operator when calculating `(total_value * i64::from(target.percentage)) / 100`. This assumes the intermediate multiplication will never exceed the 64-bit integer limit. Standard financial engines handle edge cases safely by using checked arithmetic or returning an explicit overflow error to avoid unexpected runtime panics.

✅ **Acceptance Criteria:**
- The `rebalance` function must not panic when calculating allocations for extremely large portfolio values.
- The calculation logic must be updated to handle potential arithmetic overflows safely.
- All existing tests, including chaos tests that trigger this overflow, must pass successfully after the fix.

🚫 **Out of Scope:**
- Rewriting the entire rebalancing engine to use arbitrary-precision numbers.
- Adding complex UI error dialogs for overflow states.
