1.  **Refactor `args.rs` for `plan fire` command:**
    *   Add `Plan` to `HelpTopic` enum.
    *   Add `PlanCommand` enum with a `Fire` variant.
    *   The `Fire` variant should take `monthly_expenses_cents: i64`, `liquid_assets_cents: i64`, and `monthly_savings_cents: i64`.
    *   Add `Plan(PlanCommand)` to the `Command` enum and map `Command::Plan(PlanCommand::Fire { .. })` to `"plan.fire"`.
    *   Create a `parse_plan` function that parses `plan fire` and its flags (`--monthly-expenses-cents`, `--liquid-assets-cents`, `--monthly-savings-cents`). We will use `parse_required_parsed_flag` for expenses, and `parse_optional_parsed_flag` (or similar logic) for assets and savings, defaulting them to `0` if omitted to align with the Acceptance Criteria "optionally assets".
    *   Remove `FireSim` from `AnalyticsCommand` and `parse_analytics`.
    *   Add `execute_plan_command` routing to a new `commands::plan` module.
    *   Wire `parse_plan` into `parse_args`.

2.  **Move and Rename Logic:**
    *   Create `crates/logos-cli/src/commands/plan.rs`.
    *   Move the `fire_sim` and `render_fire_sim_output` functions from `analytics.rs` to `plan.rs`, renaming `fire_sim` to `fire`.
    *   Add a test in `plan.rs` verifying the function completes.
    *   Export `pub mod plan;` in `crates/logos-cli/src/commands/mod.rs`.

3.  **Update Help Output (`help.rs`):**
    *   Add `plan fire` to `GENERAL_HELP_TEXT`.
    *   Create a `PLAN_HELP_TEXT` constant and wire it up to `HelpTopic::Plan`.
    *   Remove the old `fire-sim` help text from `ANALYTICS_HELP_TEXT`.

4.  **Polish the UI as Mosaic:**
    *   Currently, the output is formatted with `comfy_table` but lacks the "Progress %" mentioned in the Acceptance Criteria: "Must output a clear, readable summary including 'FIRE Number', 'Safe Net Worth', and 'Progress %'."
    *   In `render_fire_sim_output` (now in `plan.rs`), add a row to the summary table for `Progress %` using `sim.fire_progress_pct()`. Wait, we need to pass `fire_progress_pct` to `render_fire_sim_output`.
    *   Ensure the summary table is structured clearly and uses colors well.

5.  **Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.**

6.  **Submit the PR with the required Mosaic title and description.**
