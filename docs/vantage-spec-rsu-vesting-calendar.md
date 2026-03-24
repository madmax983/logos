# 🔭 Vantage: Spec for RSU Vesting Calendar

👤 **User Story:**
"As an employee receiving Equity Compensation, I want to see a calendar projection of my upcoming RSU vests, so that I know exactly when shares will hit my account and can plan my cash flow and tax liabilities accordingly."

💼 **Business Problem:**
Users currently struggle to visualize the timeline of their equity compensation. Without a clear calendar, they miss opportunities to plan for tax withholding or strategically distribute incoming shares. This feature turns raw RSU grants into actionable cash flow predictions.
Success = A CLI command that outputs a chronological list of upcoming vests with estimated net share counts.

✅ **Acceptance Criteria:**
- Must calculate and list future vest dates based on the grant's vesting schedule.
- Must estimate the net shares received after standard tax withholding.
- Must display the projected gross value of the vest using the current stock price.
- Must gracefully handle overlapping vests from multiple concurrent grants.

🚫 **Out of Scope:**
- Real-time stock price fetching from external APIs (users must provide a current price assumption).
- Filing actual tax returns or integrating with brokerage accounts for automatic selling.
