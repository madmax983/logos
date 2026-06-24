# 🔭 Vantage: Spec for FIRE CLI Simulator
👤 **User Story:**
As a user planning for early retirement, I want to simulate my FIRE (Financial Independence, Retire Early) trajectory via the CLI so that I can understand my safe net worth and progress without writing custom code.

🤔 **So What? (Business Problem):**
The core financial planning logic for FIRE already exists in the backend library, but it's completely inaccessible to the average user. This means our core audience cannot actually benefit from the complex projection math we've built. Exposing it via the CLI bridges this gap and transforms a backend library into a user-facing product feature.

📈 **Metric Definition:**
- Success = Users can run `logos-cli plan fire` and receive an immediate projection summary.
- Usage Metric = Number of FIRE projections run locally.

🔍 **Gap Analysis:**
Existing tools like Personal Capital or generic spreadsheets are either privacy-invasive, inflexible, or lack integration with our strict double-entry ledger. Our core library has the capability, but we lack the 'Human Interface'.

✅ **Acceptance Criteria:**
- Must expose a `plan fire` command in the CLI.
- Must accept inputs for `monthly-expenses` (and optionally assets, liabilities, and upcoming vests).
- Must output a clear, readable summary including "FIRE Number", "Safe Net Worth", and "Progress %".
- Must use existing core simulation logic.

🚫 **Out of Scope:**
- Interactive TUI screens for FIRE (Phase 2).
- Automatic pulling of assets/liabilities from the ledger (V1 will require manual input; V2 will integrate with store).
