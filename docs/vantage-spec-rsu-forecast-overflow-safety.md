# 🔭 Vantage: Spec for RSU Forecast Overflow Safety

👤 **User Story:**
"As an employee tracking a large amount of Restricted Stock Units (RSUs), I want the system to safely aggregate massive forecast values without crashing, so that edge-case grants do not break my entire reporting view."

🤔 **So What?**
A financial reporting engine must be resilient to all inputs, including extreme outliers. When the system panics on large inputs (like those near `i64::MAX`), it entirely breaks the user's ability to view their reports. A crash destroys trust in the platform's stability. Gracefully handling large numbers by capping them ensures the software remains usable even under extreme conditions.

🎯 **Metric Definition:**
Success = 0 panics during RSU forecast summaries, even when the sum of projected events exceeds `i64::MAX`. The report must generate successfully with capped values or explicit bounds errors.

🔎 **Gap Analysis:**
The `project_rsu_forecast_summary` function currently relies on the standard `.sum::<i64>()` iterator method, which panics in debug/test environments (and wraps/overflows in release) when the aggregated sum exceeds the 64-bit integer limit. Robust systems use saturating operations or safe accumulators for financial sums to prevent system crashes on edge cases.

✅ **Acceptance Criteria:**
- The `project_rsu_forecast_summary` function must not panic or cause an arithmetic overflow crash when summing large events.
- The summation logic must use safe bounds (e.g., saturating math or manual bounds checking) to prevent crashes.
- All existing tests, including `rsu_forecast_havoc`, must pass successfully.

🚫 **Out of Scope:**
- Transitioning the reporting engine to use arbitrary precision data types.
- Implementing complex UI warnings for saturated values in the CLI/TUI.
