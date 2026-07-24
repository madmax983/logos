👤 **User Story:**
As a user planning for the long term, I want to project my net worth over time via the CLI so that I can see when I'll hit my financial milestones without writing custom code.

🤔 **So What? (Business Problem):**
The Net Worth Projector is an advanced planning feature trapped in the backend library. Exposing it via the CLI allows users to actively use our projection models to plan their future milestones, turning a technical component into a valuable user-facing product feature.

📈 **Metric Definition:**
- Success = Users can run `logos-cli plan net-worth` and receive a timeline of milestones.
- Usage Metric = Number of net worth projections run locally.

🔍 **Gap Analysis:**
Users currently have to build their own spreadsheets to estimate milestone dates, which is error-prone and disconnected from their ledger data. Our core library has the native capability but lacks a user interface.

✅ **Acceptance Criteria:**
- Must expose a `plan net-worth` command in the CLI.
- Must accept inputs for `initial-net-worth`, `monthly-savings`, and optionally upcoming vests and milestones.
- Must output a chronological timeline of when milestones will be crossed.
- Must handle zero monthly savings gracefully.

🚫 **Out of Scope:**
- Automatic pulling of net worth from the ledger (V1 requires manual input; V2 will integrate with store).
- Interactive graphical charts (Phase 2).
