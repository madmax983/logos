# 🔭 Vantage: Spec for Income Router Overflow Safety

👤 **User Story:**
"As a user routing my incoming money, I want the system to safely handle extremely large income amounts without crashing, so that a massive single paycheck doesn't break the entire routing calculation."

🤔 **So What?**
A financial tool that panics on unexpected inputs is unreliable and untrustworthy. When users explore edge cases (e.g., massive salaries or inheritances), the software should gracefully report an error or cap results, rather than suffering a hard crash that destroys the entire user session.

🎯 **Metric Definition:**
Success = 0 panics during income routing, even when providing inputs like `i64::MAX`. The routing must complete and return a capped value or explicit error.

🔎 **Gap Analysis:**
Currently, `route_income` in `IncomeRouter` uses an unguarded `*` operator for calculating percentage allocations. This assumes the intermediate multiplication will never exceed the 64-bit integer limit. Standard financial simulation engines handle edge cases safely by checking for overflow.

✅ **Acceptance Criteria:**
- The `route_income` function must not panic when allocating extremely large income values.
- The allocation logic must be updated to handle potential arithmetic overflows safely.
- All existing tests, including chaos/proptests that trigger this overflow, must pass successfully after the fix.

🚫 **Out of Scope:**
- Rewriting the entire routing engine to use arbitrary-precision numbers.
