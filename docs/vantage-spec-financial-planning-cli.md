# 🔭 Vantage: Spec for Financial Planning CLI Commands

## 👤 User Story
As a user planning for long-term financial independence, I want to interactively project my net worth and simulate FIRE scenarios from the command line, so that I can easily assess my progress without writing custom code or relying on external spreadsheet models.

## 🤔 So What?
Users need to understand the trajectory of their finances, especially concerning long-term goals like FIRE (Financial Independence, Retire Early) and dealing with RSU vestings. Currently, the system supports these calculations internally, but lacks an accessible interface for users. Exposing these tools via the CLI empowers users to make data-driven financial decisions within the same ecosystem where their transaction history lives.

## 🎯 Metric Definition
Success = Users can successfully execute RSU distribution, FIRE simulation, and net worth projection commands from the terminal with a single command and receive a structured, human-readable report.

## 🔍 Gap Analysis
The system already has a robust set of financial planning engines, but they are only accessible programmatically. Standard CLI accounting tools often lack forward-looking projection capabilities, forcing users to export data to external spreadsheets to forecast long-term net worth. Adding these commands closes this loop directly within our application.

## ✅ Acceptance Criteria
- The CLI must include a command to distribute an RSU vest based on percentage policies.
- The CLI must include a command to calculate FIRE targets and safe withdrawal rates given monthly expenses and existing assets.
- The CLI must include a command to project future net worth over a specified number of months, incorporating savings and upcoming vests.
- All commands must validate user input gracefully and return clear, actionable error messages on invalid inputs.

## 🚫 Out of Scope
- Automatic synchronization of external live asset prices.
- Graphical charts or plotting within the terminal.
- Automated creation of actual ledger transactions based on projections.
