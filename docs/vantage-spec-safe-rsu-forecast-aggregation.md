# 🔭 Vantage: Spec for Safe RSU Forecast Aggregation

👤 **User Story:**
As an employee receiving large equity grants, I want to forecast massive RSU vest events without the system crashing, so that I can accurately plan for my financial future even during high-value liquidation events.

🤔 **So What?**
A forecasting engine that crashes on large, compounding RSU grants is a critical liability. High-net-worth tech employees rely on robust projections to manage their wealth. If the system panics on large inputs, it destroys user trust and renders the projection tool useless for our target demographic.

🎯 **Metric Definition:**
Success = The `project_rsu_forecast_summary` successfully processes inputs exceeding standard 64-bit bounds without panicking, safely returning a capped value (e.g., `i64::MAX`) or a clear error.

🔎 **Gap Analysis:**
Currently, `project_rsu_forecast_summary` crashes when summing projected RSU events that overflow `i64::MAX`. Competitor tools handle large projections gracefully.

✅ **Acceptance Criteria:**
- The system must handle RSU forecast summations that exceed `i64::MAX` without a runtime panic.
- A saturating mechanism or explicit failure should handle bounds gracefully.

🚫 **Out of Scope:**
- Upgrading to arbitrary-precision integers system-wide.
- Modifying non-RSU related projection flows.
