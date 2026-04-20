🔭 Vantage: Spec for Visual Cashflow Diagrams

👤 User Story
As a user tracking my personal finances, I want to generate visual flow diagrams of my income and expenses, so that I can easily understand where my money is going at a glance without reading dense tabular reports.

🤔 So What?
Ledger data is inherently dense and difficult to parse quickly. While our current tabular reports provide exact balances, they fail to effectively communicate the proportionality of spending (e.g., what percentage of my salary goes to rent vs. discretionary spending). By providing visual cashflow diagrams, we reduce the cognitive load required for financial review, making the tool more accessible and encouraging users to engage with their financial health more frequently.

🎯 Metric Definition
Success = 30% of users who run monthly reports also generate a cashflow visualization at least once a quarter.

🔍 Gap Analysis
Our current system is strictly text and table-based. Users who want to visualize their spending must export their data and build custom charts in external spreadsheet software. This breaks the seamless terminal workflow and creates a high barrier to entry for gaining visual insights. While we have internal logic capable of mapping these flows, it is not currently exposed to the user as an accessible command.

✅ Acceptance Criteria
- The application must provide a command to export cashflow data for a given time period.
- The output must be formatted as a standard diagram definition that can be rendered by popular markdown or visualization tools.
- The diagram must accurately represent the proportional flow of funds from income sources to expense categories.
- The command must gracefully handle split transactions (e.g., a single paycheck distributed across multiple savings and expense targets).
- The feature must include clear user documentation on how to render the exported diagram.

🚫 Out of Scope
- Building a custom visual rendering engine natively in the terminal.
- Live, interactive graphical user interface dashboards.
- Visualizing non-cashflow metrics (e.g., long-term net worth projections or investment performance).
