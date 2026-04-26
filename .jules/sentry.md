## 2024-04-19 - Fix test asserting missing argument value instead of flag
**Learning:** `parse_flag_value` returns `MissingArgValue` when the flag is present but the value is missing or appears to be another flag (starts with `--`). The test `rejects_import_csv_when_missing_required_flag_value` was expecting `MissingRequiredArg` instead of `MissingArgValue` due to the arguments sequence passed.
**Action:** When a flag is followed by another flag instead of a value, the parser considers the value missing for the first flag. Update tests and assertions to reflect this behavior.
## 2024-04-19 - Fix test asserting missing argument value instead of flag
**Learning:** `parse_flag_value` returns `MissingArgValue` when the flag is present but the value is missing or appears to be another flag (starts with `--`). The test `rejects_import_csv_when_missing_required_flag_value` was expecting `MissingRequiredArg` instead of `MissingArgValue` due to the arguments sequence passed.
**Action:** When a flag is followed by another flag instead of a value, the parser considers the value missing for the first flag. Update tests and assertions to reflect this behavior.
## 2024-04-20 - Portfolio Rebalancer edge cases
**Learning:** Rebalancing portfolios can hit edge cases like totally empty portfolios (`total_value <= 0`), perfectly balanced portfolios, and fractional cents from division rounding. All these paths should be explicitly tested.
**Action:** Wrote tests targeting these specific missing code paths: `test_perfectly_balanced_portfolio_returns_error`, `test_portfolio_with_zero_balance_returns_error`, `test_portfolio_rebalance_with_fractional_cents_remainder`.
## 2024-04-22 - Goal Seeker and Monte Carlo tests
**Learning:** The experimental modules `fire_goal_seeker` and `monte_carlo` lacked exhaustive tests for boundary conditions (zero target, unreachable targets, confidence level comparisons). The `FireGoalSeeker`'s `seek_monthly_contribution` might return `None` or `Some(0)` in edge cases, and the probability boundaries inside `MonteCarloProjector` required careful math bound checks. Tests should ensure we don't accidentally allocate infinite bounds or incorrect percentiles.
**Action:** Always test boundary scenarios (0 target, 0 timeframe, impossible goals) and inject deterministic math tests when dealing with probabilistic Monte Carlo engines.

## 2024-05-18 - Missing Domain Bounds Validation

**Learning:** Missing branch coverage for defensive bounds checking in domain models. While writing test for `asset_depreciation` bounds limit (division-by-zero check for 0 lifespan), similar omissions existed in bounds validations for amounts in double entry transactions. Negative amounts for debit (which expect positive values) and negative amounts for credit (which expect negative values in storage/struct logic but maybe not in initialization depending on invariant assumptions) had branches that returned errors but were never tested in `logos-core/src/domain/transaction.rs`.

**Action:** Whenever verifying bounds checks, systematically check equivalent parameter bounds checking on related functions, e.g. checking both `Posting::debit` and `Posting::credit` zero/negative boundaries. Also watch out for `cargo clippy --all-targets --all-features -- -D warnings` complaining about `.clone()` on the last use of a variable in a block of test assertions.
