# 🔭 Vantage: Spec for Interactive FIRE Dashboard (TUI)

👤 **User Story:**
As a user planning for early retirement, I want an interactive, visual dashboard in the TUI to simulate my FIRE trajectory so that I can see the impact of tweaking variables (like monthly expenses and target age) in real-time without re-running CLI commands.

🤔 **So What? (Business Problem):**
While the CLI provides a quick FIRE summary, financial planning is inherently exploratory. Users need to run "what-if" scenarios (e.g., "what if I cut expenses by 10%?"). Running repeated CLI commands is high-friction for this exploration. By providing an interactive TUI dashboard, we increase user engagement and provide immediate visual feedback on their financial trajectory, making the core projection math vastly more useful.

📈 **Metric Definition:**
- Success = Users can navigate to a FIRE view in `logos-tui`, adjust input parameters, and instantly see an updated projection graph or summary.
- Usage Metric = Time spent in the FIRE TUI dashboard vs CLI usage.

🔍 **Gap Analysis:**
The current CLI tool (`plan fire`) requires users to manually input data and re-run the command for every single scenario. Generic spreadsheets are flexible but lack the strict integration with our ledger. The `logos-tui` skeleton exists but currently lacks financial planning views.

✅ **Acceptance Criteria:**
- Must expose an interactive FIRE dashboard view within `logos-tui`.
- Must allow adjusting inputs such as monthly expenses and expected return rate using TUI controls (e.g., arrow keys).
- Must present a visual chart or table of projected net worth over time until the FIRE number is hit.
- Must reuse the existing core simulation logic.

🚫 **Out of Scope:**
- Exporting TUI projection charts to PDF or image formats (Phase 3).
- Automatic asset synchronization with external institutions for real-time portfolio updates.
