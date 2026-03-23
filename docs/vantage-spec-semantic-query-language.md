# 🔭 Vantage: Spec for Semantic Query Language

👤 **User Story:**
"As an advanced user, I want to execute semantic queries against my financial history using AletheiaDB's temporal pathfinding features, so that I can explore complex relationships (e.g., how an original transaction morphed through corrections) without having to manually sift through raw CLI logs."

💼 **Business Problem:**
A database is only as good as its query language. While we have integrated AletheiaDB and its temporal adjacency index, users currently have no way to query this rich historical data via the CLI. Exposing a semantic query interface unlocks the true value of the embedded database, turning it from a static storage backend into an analytical engine.

✅ **Acceptance Criteria:**
- **Success Metric:** Users can run a `logos-cli query` command with a semantic query string and receive formatted tabular results.
- The query interface must expose AletheiaDB's temporal pathfinding capabilities (e.g., traversing correction edges over time).
- Must fail gracefully with a clear syntax error message if the query is malformed.
- The output must be human-readable, leveraging the same dashboard-style tables used in reporting commands.

🚫 **Out of Scope:**
- Building a full graphical query builder UI.
- Direct raw SQL execution (the query language must be semantic/graph-focused as supported by AletheiaDB).
