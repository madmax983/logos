# 🔭 Vantage: Spec for RSU Forecast Gross Value Overflow Safety

👤 **User Story:**
"As an employee forecasting my RSU wealth, I want the system to safely handle extremely large stock prices or unit counts without crashing, so that a massive market shift doesn't break the entire reporting view."

🤔 **So What?**
A financial reporting engine must be resilient to all inputs, including extreme outliers in stock prices or unit counts. When the system panics on large inputs during gross value calculation, it breaks the user's ability to view their reports. A crash destroys trust in the platform's stability. Gracefully handling large numbers by capping them ensures the software remains usable even under extreme conditions.

🎯 **Metric Definition:**
Success = 0 panics during RSU `forecast_value_cents` calculation, even when providing massive `avg_close_price_cents` inputs like `i64::MAX / 50`. The function must complete and return a capped value or explicit error.

🔎 **Gap Analysis:**
Currently, `forecast_value_cents` computes a `gross_value` that, when multiplied by the `retained_pct`, uses an unguarded multiplication operator which overflows the integer bounds and panics when values are astronomically large. Standard financial simulation engines handle edge cases safely by using saturating arithmetic (capping at the maximum limit) or returning an explicit overflow error to avoid unexpected runtime panics.

✅ **Acceptance Criteria:**
- The `forecast_value_cents` function must not panic when computing the gross value with large inputs (e.g., massive stock prices or units).
- The multiplication and calculation logic must use safe bounds (e.g., saturating math or explicit checked operations) to prevent crashes.
- All existing tests, including `test_forecast_value_cents_panics_on_large_gross_value`, must pass successfully after the fix.

🚫 **Out of Scope:**
- Transitioning the core domain to use arbitrary precision data types.
- Adding complex UI warnings for saturated values in the CLI/TUI.
