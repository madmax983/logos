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
## 2026-04-27 - Arithmetic bounds and panic risks in logos-reporting
**Learning:** Found potential overflow in budget variance, net worth, cashflow, and register balance calculations. Also identified that RsuBudgetPlan used unchecked arithmetic causing negative baseline remaining budget if fixed commitments exceed base conservative budget, which should floor at 0.
**Action:** Add `.max(0)` to prevent negative `baseline_remaining_cents` and `.saturating_sub()` to all budget reporting modules where amounts subtract. Added unit tests for these boundary conditions directly.

## 2026-04-29 - Removed unsafe env modifier in tests
**Learning:** `env::set_var` in tests is intrinsically unsafe since Rust 1.80 because of multithreading environment contamination, causing undefined behavior if other tests concurrently read the environment.
**Action:** Refactored `OpCliSecretRefReader` to expose a `new(PathBuf)` constructor to allow tests to safely pass dependency paths rather than mutating global test environment state.

## 2026-04-30 - Portfolio Rebalancer edge cases
**Learning:** Missed mutants around handling remaining value distribution when `remaining_value` is `0` or fractional cents remainder are completely balanced. Wrote `should_prevent_invalid_mutants` to test `EmptyTransactionPostings` correctly and `test_negative_remaining_value_is_not_swept` to test valid edge cases. Wrote `should_not_sweep_to_target_values_if_remaining_value_is_zero_or_negative_to_kill_mutants` which covers the remainder edge conditions effectively.
**Action:** Always assert on exact posting lengths and credit/debit values using edge-case hash-map entries for boundary conditions involving fractional remainder sweeps. Note that the two remaining mutants involving `if remaining_value > 0 && !target_values.is_empty()` changing to `>= 0` or `||` are mathematically unviable/equivalent since `remaining_value` can never be negative due to `sum(percentages) == 100` and adding exactly zero to a target has no side effects.
