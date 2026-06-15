1. **Refactor UI rendering functions in `logos-tui/src/ui/reconcile.rs`**
   - Extract row creation logic from `render_runs_table` to a `fn create_run_row(...) -> Vec<Cell>` and build the rows functionally.
   - Extract row creation logic from `render_evidence_table` to a `fn create_evidence_row(...) -> Vec<Cell>` and build the rows functionally.

2. **Refactor UI rendering functions in `logos-tui/src/ui/home.rs`**
   - Extract the month metrics table building logic to `fn render_month_table(snapshot: &HomeSnapshot) -> Table`.
   - Extract the budget metrics table building logic to `fn render_budget_table(snapshot: &HomeSnapshot) -> Table`.

3. **Complete pre commit steps**
   - Complete pre commit steps to ensure proper testing, verification, review, and reflection are done.

4. **Submit PR**
   - Use the `request_code_review` tool to submit the PR titled "⚒️ Forge: Extract TUI rendering helpers" with `🚽 Smell`, `✨ Solution`, `🧼 Benefit`, `🛡️ Verification` sections.
