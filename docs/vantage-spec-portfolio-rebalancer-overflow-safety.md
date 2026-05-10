# 🔭 Vantage: Spec for Portfolio Rebalancer Overflow Safety

👤 **User Story:**
As an investor, I want the system to safely handle extremely large portfolio balances during rebalancing without crashing, so that massive asset values don't break the entire calculation.

🤔 **So What? (Business Problem):**
A financial tool that panics on unexpected inputs is unreliable and untrustworthy. When users explore edge cases (e.g., managing ultra-high net worth portfolios or using the tool for institutional sizing), the software should gracefully report an error or cap results, rather than suffering a hard crash that destroys the entire user session.

🎯 **Metric Definition:**
- Success = 0 application panics when running portfolio rebalancing with edge-case or massive portfolio values. The system must complete the rebalance safely, using saturating bounds or returning an explicit error.

🔎 **Gap Analysis:**
The current `PortfolioRebalancer` is vulnerable to arithmetic overflow during the target allocation calculation. Specifically, when computing `allocated = (total_value * target.percentage) / 100`, it fails to guard against a `total_value` large enough to exceed `i64::MAX` when multiplied by the percentage. Robust financial calculators must use safe operations (e.g., saturating arithmetic) to ensure stability.

✅ **Acceptance Criteria:**
- The `rebalance` function must not panic when computing target allocations, regardless of how large the total portfolio value is.
- Intermediate multiplication steps must be protected using saturating arithmetic to prevent overflow.
- Existing chaos tests simulating large inputs (e.g., `portfolio_rebalancer_panics_on_overflow`) must pass without triggering a panic.

🚫 **Out of Scope:**
- Rewriting the entire rebalancing engine to use arbitrary-precision numbers.
