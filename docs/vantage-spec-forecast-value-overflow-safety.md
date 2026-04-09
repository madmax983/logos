# 🔭 Vantage: Spec for Forecast Value Overflow Safety

👤 **User Story:**
"As an employee with significant equity compensation, I want the system to safely calculate the forecast value of my RSU vests even during extraordinary market conditions (extreme stock prices), so that a massive potential windfall doesn't crash the entire planning simulation."

🤔 **So What?**
A financial planning tool must remain resilient regardless of the input values. When the `forecast_value_cents` calculation panics due to astronomical stock prices, it prevents users from exploring outlier scenarios or extreme "bull cases." Crashing the application on large numbers destroys trust and makes the tool unusable for stress-testing financial models.

🎯 **Metric Definition:**
Success = 0 panics when projecting the forecast value of an RSU vest, even when inputs (like `avg_close_price_cents`) approach `i64::MAX`. The function must either return a safely capped value or fail gracefully (e.g., returning 0 or an explicit error).

🔎 **Gap Analysis:**
Currently, the forecast value calculation uses a combination of `checked_mul` and unguarded multiplication when applying the retained percentage. This assumes the intermediate gross value will never be large enough to overflow the 64-bit integer limit when multiplied by the retained percentage (e.g., 75). Robust financial systems use saturating arithmetic, safe accumulators, or complete bounds checking for all multiplication operations to prevent system crashes on edge cases.

✅ **Acceptance Criteria:**
- The forecast value calculation must not panic or cause an arithmetic overflow crash when provided with extremely large stock prices or units.
- The multiplication and percentage adjustment logic must use safe bounds (e.g., `saturating_mul`, full `checked_mul` chains, or returning 0 on overflow).
- All existing tests, including those designed to trigger overflow panics (e.g., `test_forecast_value_cents_panics_on_large_gross_value`), must pass successfully after the fix.

🚫 **Out of Scope:**
- Transitioning the domain logic to use arbitrary precision data types (e.g., `BigInt`).
- Adding complex UI error dialogs for overflow states in the CLI.