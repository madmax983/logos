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

## 2026-05-02 - CLI Presentation and Arithmetic Mutants
**Learning:** Mutants replacing CLI command bodies with `Ok(())` or altering arithmetic divisions (`/ 3`, `/ 100`) often survive if tests only verify partial outputs or lack strict numerical bounds.
**Action:** Consolidate tests into a single `mod tests` block, verify full table structures including headers, and add strict arithmetic bounds tests.
## 2026-05-02 - `budget.rs` testing and tables logic
**Learning:** Missed mutants on `comfy_table` render functions and variance calculations are because CLI string formatting operations might not be tested directly.
**Action:** Adding deterministic output validation tests checking not just the exact numeric strings but the table headers and visual bounds helps significantly improve the test suites, preventing regression in terminal presentations.

## 2026-05-02 - CLI Command Mutants
**Learning:** Mutants replacing the entire body of a CLI command with `Ok(())` frequently survive because the integration test suite (if any) is either not run by cargo mutants or doesn't actually assert the CLI output side-effects (`println!`). Testing these commands via unit tests is hard without a proper output sink dependency injected.
**Action:** Ignore UI/CLI side-effect commands (`set`, `rsu_plan`, `monte_carlo`) for strict mutation coverage unless refactoring presentation logic to return the strings instead of printing them natively.

## 2026-05-02 - CLI Presentation Mutations limits
**Learning:** `cargo mutants` often misses on presentation code where the specific presentation (like TTY colorization conditional branches using `>=`) is not evaluated by integration logic, and we shouldn't necessarily make our unit tests brittle asserting on ANSI codes just to hit coverage. `logos-cli` commands returning `Ok(())` natively represent side effects which are tricky to mock out perfectly.
**Action:** Let's stop at CLI presentation for now and consider `logos-cli` presentation tests mostly adequate since unit tests cover the core calculation logic behind them. We have strengthened the table header outputs.

## 2026-05-02 - Missing Arithmetic Coverage in Budget Planning
**Learning:** `monthly_income_cents`, `reserve_sweep_cents` calculation divisions (`/ 3`, `/ 100`) survive replacement with `%` and `*` due to weak numeric assertions.
**Action:** Covered arithmetic divisions natively.
