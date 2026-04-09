# 🔭 Vantage: Spec for Forecast Value Overflow Safety

👤 **User Story:**
"As an employee with RSU grants, I want the system to safely calculate the forecast value of my unvested shares, so that a large stock price assumption or massive number of shares does not crash the application during planning."

🤔 **So What?**
A financial modeling tool must gracefully handle edge cases. When users stress-test their financial plans with astronomical hypothetical stock prices (e.g. modelling a "moonshot" scenario), the software must not crash. A panic destroys the user session and erodes trust in the application's stability. Returning a capped value or an explicit error when theoretical limits are exceeded provides a more predictable and robust user experience.

🎯 **Metric Definition:**
Success = 0 panics during RSU forecast value calculations, even when provided with artificially massive stock prices or unit counts that would theoretically result in a gross value exceeding hardware integer limits.

🔎 **Gap Analysis:**
The system currently performs straightforward multiplication of stock prices and retained percentages during forecasting. This approach silently assumes that the result of `price * units` and the subsequent percentage calculations will never exceed standard integer bounds. Robust financial applications anticipate potential arithmetic overflows and utilize safe calculations (e.g., saturating math or explicit bounds checking) to guarantee uptime.

✅ **Acceptance Criteria:**
- The forecast value calculation must not panic when computing gross values for astronomically large inputs.
- The calculation logic must safely handle potential overflows (e.g., via saturating multiplication or returning an explicit bounds error).
- Existing test suites, specifically chaos/proptests that trigger these overflows (like `forecast_value_cents_panics_on_large_gross_value`), must pass successfully after the fix.

🚫 **Out of Scope:**
- Transitioning the core domain to use arbitrary precision data types.
- Modifying UI layers to present detailed numeric overflow warnings.
