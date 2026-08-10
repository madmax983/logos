# 🔭 Vantage: Spec for Portfolio Rebalancer Overflow Safety

👤 **User Story:**
"As an investor with a massive portfolio (or when modeling extreme economic scenarios), I want the portfolio rebalancer to calculate allocations safely, so that the system doesn't crash during rebalancing calculations due to integer overflow."

🤔 **So What? (Business Problem):**
The `PortfolioRebalancer` currently panics when rebalancing extremely large portfolio values (e.g., near `i64::MAX`) because of unguarded arithmetic multiplication (`total_value * target.percentage`). When a system crashes on extreme edge cases, it erodes user trust and prevents reliable extreme-value financial modeling or simulations.

📈 **Metric Definition:**
- Success = 0 panics during `rebalance` when `total_value` approaches `i64::MAX`. The system must complete the balancing calculation or return a clean error without panicking.

🔍 **Gap Analysis:**
Currently, `PortfolioRebalancer::rebalance` crashes with an `attempt to multiply with overflow` panic when processing extremely large values, as demonstrated by the `portfolio_rebalancer_panics_on_overflow` test. Standard financial processors use safe bounds or saturating arithmetic for intermediate percentage calculations to avoid this issue.

✅ **Acceptance Criteria:**
- The `PortfolioRebalancer::rebalance` method must not panic when computing target allocations for extremely large portfolio balances.
- Intermediate multiplication steps must be protected against arithmetic overflow (e.g. using saturating/checked math or re-ordering operations).

🚫 **Out of Scope:**
- Upgrading the entire underlying storage engine to arbitrary precision integers.
- Implementing automated trading features or brokerage API integrations.
