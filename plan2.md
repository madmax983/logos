1.  **Refactor `args.rs` for `plan fire` command:**
    *   Add `Plan` to `HelpTopic` enum.
    *   Add `PlanCommand` enum with a `Fire` variant.
    *   The `Fire` variant should take `monthly_expenses_cents: i64`, `liquid_assets_cents: i64`, and `monthly_savings_cents: i64`.
    *   Add `Plan(PlanCommand)` to the `Command` enum and map `Command::Plan(PlanCommand::Fire { .. })` to `"plan.fire"`.
    *   Create a `parse_plan` function that parses `plan fire` and its flags. Use `parse_required_parsed_flag` for `--monthly-expenses-cents`. Use `parse_optional_parsed_flag` for `--liquid-assets-cents` and `--monthly-savings-cents`, defaulting both to `0`.
    *   Remove `FireSim` from `AnalyticsCommand` and `parse_analytics`.
    *   Add `execute_plan_command` routing to a new `commands::plan` module.
    *   Wire `parse_plan` into `parse_args`.

2.  **Move and Rename Logic:**
    *   Create `crates/logos-cli/src/commands/plan.rs`.
    *   Move the `fire_sim` and `render_fire_sim_output` functions from `analytics.rs` to `plan.rs`, renaming `fire_sim` to `fire`.
    *   Add the following test in `plan.rs`:
        ```rust
        #[cfg(test)]
        mod fire_sim_tests {
            use super::*;
            #[test]
            fn test_fire_sim_calculates_correctly() {
                let result = fire(500_000, 1_000_000, 200_000);
                assert!(result.is_ok());
            }
        }
        ```
    *   Export `pub mod plan;` in `crates/logos-cli/src/commands/mod.rs`.

3.  **Verify New File Creation:**
    *   Use `read_file` on `crates/logos-cli/src/commands/plan.rs` to confirm the file was created and populated correctly.

4.  **Update Help Output (`help.rs`):**
    *   Add `plan fire` to `GENERAL_HELP_TEXT`.
    *   Create a `PLAN_HELP_TEXT` constant and wire it up to `HelpTopic::Plan` in `help_text`. The help text should be:
        ```text
        Usage: ledger plan <subcommand> [options]

        Subcommands:
          fire --monthly-expenses-cents <i64> [--liquid-assets-cents <i64>] [--monthly-savings-cents <i64>]
                                             Simulate time to Financial Independence
        ```
    *   Remove the old `fire-sim` help text from `ANALYTICS_HELP_TEXT`.

5.  **Polish the UI as Mosaic:**
    *   The `FireSimulator` in `logos-core` has a `fire_progress_pct()` method.
    *   In `plan.rs`, change `render_fire_sim_output` to accept `progress_pct: u8`.
    *   Update `fire` function in `plan.rs` to calculate `let progress_pct = sim.fire_progress_pct();` and pass it to `render_fire_sim_output`.
    *   Add a new row in `render_fire_sim_output` table for "Progress %" formatted as `{progress_pct}%`, using `comfy_table::Color::Yellow` for the color.

6.  **Run tests, linters, and formatters:**
    *   Run `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, and `cargo fmt --all`.

7.  **Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.**

8.  **Submit the PR:**
    *   Submit the change with a PR structured according to the "Mosaic" persona:
```markdown
🎨 Mosaic: UI Polish for FIRE CLI

🖌️ **Before:**
The FIRE logic was hidden under `analytics fire-sim`, which outputted a basic table without showing the overall Progress Percentage. In addition, `liquid-assets-cents` and `monthly-savings-cents` were strictly required, conflicting with product specs.

✨ **After:**
The logic has been promoted to a primary `plan fire` command. The command gracefully defaults optional parameters to `0`. The output table now explicitly includes a highlighted "Progress %" row, providing immediate feedback on how far along the user is on their FIRE journey.

🖼️ **Visuals:**
The CLI now acts like a mini-dashboard for retirement planning, with a clear separation between Current Status (Expenses, Net Worth, Savings, Progress %) and the Milestone Journey table.
```
