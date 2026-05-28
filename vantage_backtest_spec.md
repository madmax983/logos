# Vantage: Spec for Backtest Command

👤 **User Story:**
As a Trader, I want to backtest against volatile markets so that I can validate my allocation strategy.

🤔 **So What? (Business Problem):**
Users have no way to test their strategies against historical volatility.

📈 **Metric Definition:**
- Success = Query latency < 10ms for 99% of requests.

🔍 **Gap Analysis:**
Standard libs lack backtesting.

✅ **Acceptance Criteria:**
- Must handle NaN data without panicking.
- Must output a CSV report.

🚫 **Out of Scope:**
- Real-time execution (Phase 2).
