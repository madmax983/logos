# 🔭 Vantage: Spec for CLI Planning Namespace

👤 **User Story:**
As a user performing financial planning, I want a dedicated `plan` command namespace in the CLI so that I can easily discover and execute forward-looking projections without confusing them with retroactive `analytics`.

🤔 **So What? (Business Problem):**
Currently, forward-looking financial planning tools like `fire-sim` are nested under the `analytics` command. This creates a confusing User Experience, as `analytics` implies looking at past or current data, whereas `fire-sim` projects future outcomes. Grouping planning features under a dedicated `plan` namespace improves feature discoverability and sets clear mental models for the user.

📈 **Metric Definition:**
- Success = The `fire-sim` command is successfully moved to `logos-cli plan fire`.
- Usage Metric = Number of times commands under the `plan` namespace are invoked.

🔍 **Gap Analysis:**
The current CLI structure lumps projections (`fire-sim`) and data exports (`snapshot create`) together under `analytics`. We need to delineate "What happened?" (`analytics`) from "What could happen?" (`plan`).

✅ **Acceptance Criteria:**
- Must introduce a new `plan` command to the CLI.
- Must move the existing `fire-sim` command from `analytics fire-sim` to `plan fire`.
- Must preserve all existing input arguments and simulation output logic for the moved commands.
- Must update help text to reflect the new `plan` namespace.

🚫 **Out of Scope:**
- Interactive TUI screens for planning (Phase 2).
- Automatic pulling of assets/liabilities from the ledger for planning inputs (Phase 2).
