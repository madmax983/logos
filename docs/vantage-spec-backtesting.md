# 🔭 Vantage: Spec for Backtesting Simulator

👤 **User Story:** "As a Trader, I want to backtest against volatile markets so that I can understand my safe net worth and progress without writing custom code."

🤔 **So What? (Business Problem):**
The core financial planning logic for FIRE already exists in the backend library, but it lacks the ability to simulate past market volatility. Exposing a backtesting feature bridges this gap and transforms our simulation logic into a robust, trust-building tool for users evaluating their financial resilience.

📈 **Metric Definition:**
- Success = Users can run the backtesting simulator and receive a CSV report containing the results.

🔍 **Gap Analysis:**
Existing tools do not integrate well with our strict double-entry ledger or fail to handle NaN data securely. We need a simulator that natively supports volatile market data while ensuring system stability.

✅ **Acceptance Criteria:**
- Must handle NaN data without panicking.
- Must output a CSV report.

🚫 **Out of Scope:** Real-time execution (Phase 2).
