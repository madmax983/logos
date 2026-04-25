# 🔭 Vantage: Spec for Mermaid Diagram Exporter

👤 **User Story:**
"As a user visualizing my complex cashflows and transactions, I want to automatically generate Mermaid.js diagram code from my ledger data, so that I can easily view my financial flows in external documentation tools like GitHub or Notion without drawing them by hand."

💼 **Business Problem:**
Double-entry ledgers are extremely precise but visually dense and hard for humans to parse quickly. A visual diagram like a Sankey flow or an XY chart immediately communicates where money is coming from and where it's going. The application currently lacks a formal product spec to guide its integration into user-facing commands (like an `analytics export` CLI command).

🎯 **Metric Definition:**
Success = The exporter can reliably take a set of standard ledger transaction records and output syntactically valid Mermaid.js markdown strings that render correctly in standard markdown viewers.

🔍 **Gap Analysis:**
Currently, the codebase contains experimental diagram generation modules, but there is no overarching spec that defines the product requirement or guides how these exporters should be consumed by the CLI (e.g., as part of an `analytics sankey` or similar command).

✅ **Acceptance Criteria:**
- Must support exporting data in a format suitable for Sankey diagrams (showing flow between accounts).
- Must support exporting data in a format suitable for XY charts (showing balance over time).
- Must ensure the generated string is valid Mermaid markdown (e.g., correctly escaping account names if they contain special characters).
- Must handle both positive (debit) and negative (credit) flows accurately in the visual representation.

🚫 **Out of Scope:**
- Building a built-in interactive visual renderer (the output is strictly text/markdown to be pasted into other tools).
- Exporting to non-Mermaid diagram formats (e.g., Graphviz).
