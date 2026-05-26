# 🔭 Vantage: Spec for Portfolio Rebalancer Overflow Safety

👤 **User Story:**
"As an investor, I want the system to safely handle extremely large portfolio balances during rebalancing without crashing, so that massive inputs don't break the entire application."

🤔 **So What?**
A simulation tool that panics on unexpected inputs is unreliable and untrustworthy. When users explore edge cases (e.g., extremely large portfolios), the software should gracefully cap results at hardware limits or report an error, rather than suffering a hard crash that destroys the entire user session. Resilient software builds trust; fragile software creates frustration.

🎯 **Metric Definition:**
Success = 0 panics during portfolio rebalancing, even when providing inputs like `i64::MAX`. The simulation must complete and return an explicit error or cap safely.

🔎 **Gap Analysis:**
Currently, `rebalance` uses an unguarded `*` operator when computing the allocation percentage. This assumes the portfolio value multiplied by the percentage will never exceed the 64-bit integer limit. Standard financial simulation engines handle edge cases safely by using saturating arithmetic (capping at the maximum limit) or returning an explicit overflow error to avoid unexpected runtime panics.

✅ **Acceptance Criteria:**
- The `rebalance` function must not panic when computing allocations for extremely large values.
- The allocation logic must be updated to handle potential arithmetic overflows safely.
- All existing tests, including chaos/proptests that trigger this overflow, must pass successfully after the fix.

🚫 **Out of Scope:**
- Rewriting the entire portfolio rebalancer logic.
- Adding complex UI error dialogs for overflow states.
