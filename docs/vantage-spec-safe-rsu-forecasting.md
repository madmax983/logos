# 🔭 Vantage: Spec for Safe RSU Forecasting

👤 **User Story:**
As an employee with high-value equity compensation, I want to forecast massive Restricted Stock Unit (RSU) events without the application crashing, so that I can reliably predict my future net worth during major liquidity events.

🤔 **So What?**
If the reporting engine crashes on large valuations, high net worth users lose trust in the tool. Reliable large-number handling ensures stability for our most valuable target demographic.

🎯 **Metric Definition:**
Success = The RSU forecast calculation successfully handles extremely large aggregate valuations with zero system panics, either by graceful capping or explicit error messaging.

🔎 **Gap Analysis:**
Currently, large RSU forecasts crash the reporting system due to arithmetic limits. Standard financial software handles this gracefully to avoid unexpected runtime crashes.

✅ **Acceptance Criteria:**
- The system must handle large equity compensation aggregations that exceed standard data limits without panicking.
- The user must receive a clear error message or a saturated/capped value instead of a system crash.

🚫 **Out of Scope:**
- Upgrading the entire underlying storage engine to arbitrary precision numbers.
- Implementation details such as specific structs, types, or methods used to fix the overflow.
