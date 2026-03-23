# 🔭 Vantage: Spec for Monte Carlo Investment Simulator

👤 **User Story:**
"As a long-term investor, I want to project a range of possible future net worth outcomes using randomized market returns, so that I can understand the probability of reaching my financial goals under different market conditions."

💼 **Business Problem:**
Deterministic projections (like a flat 7% return) create a false sense of certainty and fail to account for market volatility. Users need probabilistic models to make resilient financial plans. Success = A CLI command that outputs the 5th, 50th, and 95th percentile net worth outcomes over a specified horizon.

✅ **Acceptance Criteria:**
- Must simulate multiple paths (e.g., 1000+) using an expected return and volatility.
- Must output the 5th, 50th (median), and 95th percentile outcomes in cents.
- Must avoid heavy external dependencies (e.g., no `rand` crate) to keep the core library lightweight and deterministic.

🚫 **Out of Scope:**
- Interactive visual graphs (CLI output is text/table only).
- Real-time fetching of historical asset volatility (users must provide expected return/volatility parameters).