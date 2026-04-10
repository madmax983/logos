# 🔭 Vantage: Spec for RSU Forecast Value Overflow Safety

👤 **User Story:**
"As an employee analyzing my future Restricted Stock Units (RSUs), I want the system to safely handle extremely large forecasted stock prices or unit counts without crashing, so that a large initial value or projected growth rate does not break my entire portfolio view."

🤔 **So What?**
Financial forecasting tools must gracefully handle extreme inputs. Currently, the `forecast_value_cents` calculation panics if a very large estimated stock price (e.g., `i64::MAX / 50`) is multiplied by a retention percentage (e.g., 75%). This causes the entire system to crash. When software panics on out-of-bounds calculations rather than safely capping the result or returning an explicit error, it destroys user confidence and halts the reporting engine. Complexity is a cost; stability is utility.

🎯 **Metric Definition:**
Success = 0 panics during RSU value forecasting, even when processing gross values that would normally cause an arithmetic overflow upon multiplying by the retention percentage. The function must execute safely and cap results at the maximum limit or return an explicit bounds error.

🔎 **Gap Analysis:**
The `forecast_value_cents` function calculates `gross_value` and then multiplies it by a percentage (e.g., `retained_pct`) before dividing by 100. This intermediate multiplication is currently unguarded. Standard financial engines prevent integer overflow during percentage calculations by either using saturating arithmetic, larger intermediate types (like `i128`), or safe division-first algorithms.

✅ **Acceptance Criteria:**
- The `forecast_value_cents` calculation must not panic when computing the retained value from an extremely large `gross_value`.
- The arithmetic must be updated to use safe bounds (such as saturating multiplication, `i128` intermediates, or safe sequence of operations) to prevent crashes.
- All existing tests, including `test_forecast_value_cents_panics_on_large_gross_value` that triggers this crash, must pass successfully after the update (e.g. they should check for capped values rather than expecting panics).

🚫 **Out of Scope:**
- Transitioning the core domain to arbitrary precision numbers for all calculations.
- Implementing complex UI dialogs in the CLI or TUI to warn the user about theoretical bounds limits.
