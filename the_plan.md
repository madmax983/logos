1. **Create `PlanCommand` and move `fire_sim` logic to a new `plan` module.**
   - In `crates/logos-cli/src/args.rs`:
     - Add `Plan` to `HelpTopic` enum.
     - Add `PlanCommand` enum with `Fire` variant: `Fire { monthly_expenses_cents: i64, liquid_assets_cents: i64, monthly_savings_cents: i64 }`.
     - Remove `FireSim` from `AnalyticsCommand` enum.
     - Add `Plan(PlanCommand)` to `Command` enum.
     - Update `Command::path` for `Plan(PlanCommand::Fire) => "plan.fire"`.
     - Update `execute_command` to handle `Command::Plan(command) => execute_plan_command(command)`.
     - Add `fn execute_plan_command` calling `commands::plan::fire`.
     - Remove `fire-sim` from `parse_analytics`.
     - Add `parse_plan` function to handle `logos-cli plan fire` parsing with identical flags, but make `liquid-assets-cents` and `monthly-savings-cents` optional with default 0 if not provided.
     - Also add `plan` to `parse_args` matching.

2. **Move CLI command implementation.**
   - Move `pub fn fire_sim` and `fn render_fire_sim_output` from `crates/logos-cli/src/commands/analytics.rs` to a new `crates/logos-cli/src/commands/plan.rs`.
   - Rename to `pub fn fire`.
   - Update `crates/logos-cli/src/commands/mod.rs` to export `plan`.

3. **Update Help Output.**
   - In `crates/logos-cli/src/commands/help.rs`:
     - Update `GENERAL_HELP_TEXT` to include `plan fire` command.
     - Create `PLAN_HELP_TEXT`.
     - Update `ANALYTICS_HELP_TEXT` to remove `fire-sim`.

4. **Verify tests.**
   - Run tests and fix any failing cases.

5. **Submit.**
