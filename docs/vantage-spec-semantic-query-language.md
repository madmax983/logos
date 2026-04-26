# 🔭 Vantage: Spec for Semantic Query Language

👤 **User Story:**
"As a power user, I want to be able to query my transactions and balances using a domain-specific semantic query language, so that I can quickly slice and dice my financial data without writing raw SQL or exporting to Excel."

🤔 **So What?**
A database is only as good as its query language. If users cannot easily filter and aggregate their data based on financial dimensions (e.g., date ranges, accounts, categories, amounts), the ledger is just a black box. Providing a semantic query language empowers users to extract maximum utility from their historical data directly within the CLI/TUI.

🎯 **Metric Definition:**
Success = Users can query transactions by account, date range, amount, and text content directly from the CLI or TUI. 95% of queries execute in under 100ms.

🔎 **Gap Analysis:**
Currently, users are limited to hardcoded reports (e.g., month, register) or exporting to CSV. There is no dynamic querying capability. We need a parser and execution engine for a simple semantic query language (e.g., `account:expenses amount:>100 date:2024-01..2024-12`).

✅ **Acceptance Criteria:**
- Must support filtering by account (`account:X`), amount (`amount:>X`, `amount:<X`), and date ranges (`date:X..Y`).
- Must support substring matching on transaction descriptions.
- The query language must be available via a new CLI command (`logos-cli query "..."`).
- Queries must run against the production Postgres database using parameterized inputs to prevent injection.

🚫 **Out of Scope:**
- Complex JOINs or multi-table aggregations.
- Natural language processing (NLP) to parse plain English questions.
