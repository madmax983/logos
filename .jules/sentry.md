## 2024-04-19 - Fix test asserting missing argument value instead of flag
**Learning:** `parse_flag_value` returns `MissingArgValue` when the flag is present but the value is missing or appears to be another flag (starts with `--`). The test `rejects_import_csv_when_missing_required_flag_value` was expecting `MissingRequiredArg` instead of `MissingArgValue` due to the arguments sequence passed.
**Action:** When a flag is followed by another flag instead of a value, the parser considers the value missing for the first flag. Update tests and assertions to reflect this behavior.
## 2024-04-19 - Fix test asserting missing argument value instead of flag
**Learning:** `parse_flag_value` returns `MissingArgValue` when the flag is present but the value is missing or appears to be another flag (starts with `--`). The test `rejects_import_csv_when_missing_required_flag_value` was expecting `MissingRequiredArg` instead of `MissingArgValue` due to the arguments sequence passed.
**Action:** When a flag is followed by another flag instead of a value, the parser considers the value missing for the first flag. Update tests and assertions to reflect this behavior.
## 2024-04-20 - Portfolio Rebalancer edge cases
**Learning:** Rebalancing portfolios can hit edge cases like totally empty portfolios (`total_value <= 0`), perfectly balanced portfolios, and fractional cents from division rounding. All these paths should be explicitly tested.
**Action:** Wrote tests targeting these specific missing code paths: `test_perfectly_balanced_portfolio_returns_error`, `test_portfolio_with_zero_balance_returns_error`, `test_portfolio_rebalance_with_fractional_cents_remainder`.
