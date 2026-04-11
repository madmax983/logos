# 🔭 Vantage: Spec for Safe RSU Forecasting

## 👤 User Story
As a user tracking my RSU vesting, I want the system to safely handle unusually large stock prices or unit counts without crashing, so that edge-case grants or massive market valuations do not break my entire reporting view.

## 🤔 So What? (Business Problem)
A financial forecasting engine must be resilient to all inputs, including extreme outliers. When the system panics on large inputs (e.g. `i64::MAX / 50` for stock price), it entirely breaks the user's ability to view their reports. A crash destroys trust in the platform's stability. Gracefully handling large numbers by capping them ensures the software remains usable even under extreme conditions.

## 📈 Metric Definition
- **Success Criteria**: 0 panics during RSU forecasting, even when processing extreme values for `avg_close_price_cents` or `units`. The function must complete and return a capped value or explicit error.

## 🔍 Gap Analysis
- **Current State**: The system panics when calculating the forecast value of RSUs given extremely large values, causing a hard crash. The application assumes that values will not reach high enough levels to cause arithmetic overflow bounds errors.
- **Competitors**: Standard financial simulation engines handle edge cases safely by using saturating arithmetic (capping at the maximum limit) or returning an explicit overflow error to avoid unexpected runtime panics.

## ✅ Acceptance Criteria
- The system must not panic or crash when computing the forecasted value of RSUs with large inputs.
- The `forecast_value_cents` calculation must use safe arithmetic bounds (e.g., saturating operations or returning an error/capped value) to prevent crashes.
- All existing tests, including `test_forecast_value_cents_panics_on_large_gross_value`, must pass successfully.

## 🚫 Out of Scope
- Transitioning the reporting engine to use arbitrary precision data types.
- Complex UI warnings for saturated values in the CLI/TUI.