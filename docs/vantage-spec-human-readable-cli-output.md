# 🔭 Vantage: Spec for Human-Readable CLI Output

👤 **User Story:**
"As an everyday user of Logos, I want to see simple, clear status messages when running basic commands like setting a budget or running a report, so that I understand what the system is doing without needing to parse complex technical database jargon."

💼 **Business Problem:**
The application is confusing new and non-technical users by exposing internal database implementation details ("temporal adjacency index", "Index restoration completed successfully") in the standard CLI output. This technical jargon increases perceived complexity, creates friction during onboarding, and damages the user's trust in the tool as a simple personal finance solution. Complexity is a cost; clear communication is utility.

✅ **Acceptance Criteria:**
- **Success Metric:** Standard, non-error CLI output for basic commands (`budget set`, `report month`, etc.) contains zero instances of internal database jargon (e.g., "temporal", "adjacency", "index", "restoration").
- The system must display simplified, human-readable status messages during startup/loading phases (e.g., "Loading history", "Database loaded").
- Technical and diagnostic messages must be hidden by default during standard execution.
- If deep technical logs are required for debugging, they must be gated behind an explicit verbose or debug flag (e.g., `-v`, `--debug`, or via `RUST_LOG`).

🚫 **Out of Scope:**
- Rewriting the underlying storage engine or altering how the indices actually function.
- Changing the structure or format of actual financial data output (e.g., the content of the reports themselves).
- Building a full graphical user interface (this scope is strictly limited to CLI text output).
