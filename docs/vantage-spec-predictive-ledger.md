# 🔭 Vantage: Spec for Predictive Ledger

## 👤 User Story
As a planner managing my future cash flow, I want the system to generate a projected ledger of expected future transactions based on my recurring templates, so that I can see what my balance will likely be at specific future dates.

## 🤔 So What? (Business Problem)
Traditional ledgers only show the past. Financial planning requires looking into the future. A predictive ledger transforms the tool from a historical record-keeper into a forward-looking cash flow management system, allowing users to anticipate shortfalls before they happen.

## 📈 Metric Definition
Success = The system can generate simulated ledger entries for future dates based on established recurrence templates and merge them visually with current balances.

## 🔍 Gap Analysis
Users currently have to project their future balances in their heads or in external spreadsheets. While we can detect recurring cashflows, we lack a formal "future ledger" view that projects those cashflows onto a timeline.

## ✅ Acceptance Criteria
- Must take existing account balances and a list of recurring templates.
- Must project future transaction events based on the templates' frequency rules.
- Must calculate the running balance of accounts as these projected transactions "occur."
- Must output a chronological ledger combining actual historical transactions and projected future ones.

## 🚫 Out of Scope
- Simulating macroeconomic shocks or inflation on future cashflows.
- Automatically executing these future transactions against real bank accounts.
