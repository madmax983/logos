👤 **User Story:**
As a user focused on long-term wealth building, I want to project my net worth trajectory and milestone dates via the CLI, so that I can visualize when I will reach my financial goals without building manual spreadsheets.

🤔 **So What? (Business Problem):**
We have a powerful `Net Worth Projector` in our core library, but users cannot access it. Users currently have to guess when they will hit major financial milestones. Exposing this via the CLI allows users to quickly simulate how their monthly savings and upcoming stock vests accelerate their path to wealth, turning complex backend logic into actionable financial insight.

📈 **Metric Definition:**
- Success = Users can run `logos-cli plan net-worth` and see a timeline of their projected wealth and milestone dates.
- Usage Metric = Number of net worth projections executed.

🔍 **Gap Analysis:**
Most budgeting tools look backwards. Our double-entry ledger is strict and historical. By exposing this projector, we bridge the gap between historical accounting and forward-looking financial planning, offering a unique capability that standard CLI ledgers lack.

✅ **Acceptance Criteria:**
- Must expose a `plan net-worth` command in the CLI.
- Must accept inputs for `initial-net-worth`, `monthly-savings`, and `months` to project.
- Must allow inputting target milestones (e.g., `--milestone 1000000`).
- Must output a timeline of projected net worth and a list of when milestones are crossed.
- Must use the existing `NetWorthProjector` core logic.

🚫 **Out of Scope:**
- Automatic extraction of `initial-net-worth` from the ledger (V1 will require manual input).
- Interactive charting or TUI interfaces (Phase 2).
