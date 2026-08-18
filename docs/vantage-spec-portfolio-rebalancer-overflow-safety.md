# 🔭 Vantage: Spec for Portfolio Rebalancer Resilience

👤 **User Story:**
As an investor rebalancing my portfolio, I want the system to safely handle extremely large balances without crashing, so that massive portfolio amounts don't break the entire rebalancing calculation.

🤔 **So What?**
A financial tool that panics on unexpected inputs is unreliable and untrustworthy. When users explore edge cases (e.g., extremely large portfolio balances), the software should gracefully report an error or handle the math safely, rather than suffering a hard crash that destroys the entire user session.

📈 **Metric Definition:**
Success = 0 panics during portfolio rebalancing, even when providing inputs like `i64::MAX`. The rebalancing must complete or explicitly return an explicit bounding error.

🔎 **Gap Analysis:**
Currently, the `rebalance` logic in the portfolio rebalancer uses unguarded arithmetic multiplication (`*`) for calculating percentage allocations. This assumes the intermediate multiplication will never exceed the 64-bit integer limit. Standard financial engines handle these edge cases safely by explicitly checking for overflow.

✅ **Acceptance Criteria:**
- The portfolio rebalancer logic must not panic when rebalancing extremely large values.
- The rebalancer must handle potential arithmetic overflows safely by catching them and returning an error or capping the result.
- All existing tests, including chaos/proptests that trigger this overflow, must pass successfully after the fix.

🚫 **Out of Scope:**
- Transitioning to arbitrary precision integers for all portfolio calculations.
