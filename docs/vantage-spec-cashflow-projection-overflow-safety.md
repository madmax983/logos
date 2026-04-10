# 🔭 Vantage: Spec for Cashflow Projection Overflow Safety

👤 **User Story:**
"As a user planning my financial future, I want the system to safely handle extremely large cashflow projections without crashing, so that a massive single input or long-term compounding effect doesn't break the entire simulation."

🤔 **So What?**
A simulation tool that panics on unexpected inputs is unreliable and untrustworthy. When users explore edge cases (e.g., massive salaries, extreme compounding over decades), the software should gracefully cap results at hardware limits or report an error, rather than suffering a hard crash that destroys the entire user session. Resilient software builds trust; fragile software creates frustration.

🎯 **Metric Definition:**
Success = 0 panics during cashflow simulation, even when providing inputs like `i64::MAX`. The simulation must complete and return a capped value or explicit error.

🔎 **Gap Analysis:**
Currently, `project_balances` uses an unguarded `+=` operator for aggregating cashflows. This assumes the sum will never exceed the 64-bit integer limit. Standard financial simulation engines handle edge cases safely by using saturating arithmetic (capping at the maximum limit) or returning an explicit overflow error to avoid unexpected runtime panics.

✅ **Acceptance Criteria:**
- The `project_balances` function must not panic when aggregating extremely large cashflow values.
- The aggregation logic must be updated to handle potential arithmetic overflows safely (e.g., via saturating addition).
- All existing tests, including chaos/proptests that trigger this overflow, must pass successfully after the fix.

🚫 **Out of Scope:**
- Rewriting the entire projection engine to use arbitrary-precision numbers.
- Adding complex UI error dialogs for overflow states.
