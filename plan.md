1. **Explore the codebase and understand the requirements:**
   - The user (as "Vantage") wants to add a CLI feature to run FIRE (Financial Independence, Retire Early) and net worth projections.
   - The specs mention a new `plan fire` command for `logos-cli`.
   - The command should compute FIRE numbers and safe net worth using `logos_core::fire::FireSimulator`.
   - The command should accept `--monthly-expenses-cents` and optionally `--safe-withdrawal-rate-pct`, `--liquid-assets-cents`, and `--liabilities-cents`.
   - Also, a `plan project` command should be added as an optional feature to project net worth milestones.

2. **Implementation details:**
   - I have already modified `crates/logos-cli/src/args.rs` to parse the new commands (`plan fire` and `plan project`) using existing patterns.
   - I added `crates/logos-cli/src/commands/plan.rs` to implement the logic for `fire` and `project` using `FireSimulator` and `NetWorthProjector` from `logos_core`.
   - Output uses `comfy-table` for tabular reporting, formatting the cents as currency.
   - I updated the help text in `crates/logos-cli/src/commands/help.rs` and added the `plan` subcommand to the `HelpTopic`.
   - The unit tests in `crates/logos-cli/tests/cli_parse.rs` have been updated and are passing.

3. **Pre-commit checks:**
   - Run `cargo fmt --all`.
   - Run `cargo clippy --workspace --all-targets -- -D warnings`.
   - Run `cargo test --workspace`.
   - All tests pass, and clippy warnings were fixed (including fixing `implicit_clone` warnings in tests in `logos-tui`).
   - Call `pre_commit_instructions` tool to make sure proper testing, verifications, reviews and reflections are done.

4. **Submit changes:**
   - Commit the changes and open a PR with the title and description aligned with the vantage specs.
   - Use the `submit` tool to push the branch.
