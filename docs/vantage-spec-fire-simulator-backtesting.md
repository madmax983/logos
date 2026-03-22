# 🔭 Vantage: Spec for FIRE Simulator Backtesting

👤 **User Story:**
"As a Trader, I want to backtest against volatile markets, so that I can understand how my FIRE (Financial Independence, Retire Early) target and safe net worth progress hold up under historical stress scenarios."

💼 **Business Problem:**
Users need confidence in their FIRE targets and safe net worth progress when facing volatile markets. Backtesting provides that confidence and makes the tool more valuable for long-term planning. Success = A robust CSV report output without panicking on bad data.

✅ **Acceptance Criteria:**
- Must handle NaN data without panicking.
- Must output a CSV report containing the historical scenario inputs and the resulting FIRE progress metrics.
- Must accept a series of historical market variance multipliers to apply against the configured `FireConfig` and `UpcomingVest` inputs.

🚫 **Out of Scope:**
- Real-time execution (Phase 2).
