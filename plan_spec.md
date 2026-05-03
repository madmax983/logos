1. **Move logic from `analytics fire-sim` to `plan fire`.**
   - The user story says `logos-cli plan fire` is the goal.
   - We need to create a new `PlanCommand` enum, `parse_plan` function in `args.rs`.
   - The CLI should handle `plan fire` with arguments similar to `analytics fire-sim` today, but we may also allow optional arguments as specified in the User Story (inputs for `monthly-expenses` and optionally assets, liabilities, upcoming vests).
   - The current `analytics fire-sim` arguments are: `--monthly-expenses-cents`, `--liquid-assets-cents`, `--monthly-savings-cents`. Wait, the requirement says "Must accept inputs for monthly-expenses (and optionally assets, liabilities, and upcoming vests)". Let's look closely at `logos_core::fire::FireSimulator`.

Wait, the prompt says "Move logic from analytics fire-sim to plan fire"? No, the prompt is:
"As a user planning for early retirement, I want to simulate my FIRE (Financial Independence, Retire Early) trajectory via the CLI so that I can understand my safe net worth and progress without writing custom code.
✅ Acceptance Criteria:
- Must expose a `plan fire` command in the CLI.
- Must accept inputs for `monthly-expenses` (and optionally assets, liabilities, and upcoming vests).
- Must output a clear, readable summary including "FIRE Number", "Safe Net Worth", and "Progress %".
- Must use existing core simulation logic.
"
