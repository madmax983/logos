# 🔭 Vantage: Spec for RSU Forecast Value Overflow Safety

👤 **User Story:**
"As a user tracking my financial future, I want the system to safely compute the projected value of my RSU vests even with massive stock prices, so that extreme market events or data anomalies don't cause the application to crash."

🤔 **So What?**
A financial tool that panics on unexpectedly large numbers is fragile and destroys user trust. If the forecasting engine crashes due to an arithmetic overflow on a massive RSU grant, the entire reporting view becomes inaccessible. Graceful handling of extreme edge cases (like capping values or safely evaluating to zero) ensures the application remains robust, reliable, and continuously usable.

🎯 **Metric Definition:**
Success = 0 panics during RSU forecast value computation (`forecast_value_cents`), even when provided with inputs that result in astronomically large gross values (e.g., `avg_close_price_cents` near `i64::MAX`). The system must complete execution and return a safe fallback value (like 0) instead of crashing.

🔎 **Gap Analysis:**
Currently, when computing the forecast value of an RSU vest, an extremely large gross value multiplied by the retained percentage can exceed the bounds of a 64-bit integer. Standard arithmetic operations or unguarded multiplications will panic in these overflow scenarios. Resilient financial systems avoid this by utilizing checked math operations (e.g., `checked_mul`) and safely falling back on bounds rather than crashing.

✅ **Acceptance Criteria:**
- The `forecast_value_cents` function must not panic or cause an arithmetic overflow crash when computing the adjusted value of massive gross amounts.
- The multiplication and percentage adjustment logic must employ safe bounds checking (like returning 0 on overflow).
- All tests, including chaotic inputs designed to trigger overflow, must pass successfully without panicking.

🚫 **Out of Scope:**
- Upgrading the entire financial engine to use arbitrary-precision numbers.
- Adding complex UI error dialogs or warnings for overflowed forecast values.
