# 🔭 Vantage: Spec for FIRE TUI Dashboard

## 👤 User Story
As a user planning for Financial Independence, Retire Early (FIRE),
I want an interactive dashboard in the TUI to simulate my FIRE trajectory,
so that I can quickly explore different scenarios without repeatedly typing CLI commands.

## 🤔 So What? (Business Problem)
Financial planning is inherently exploratory. Users need a tight feedback loop to answer questions like "What if my expenses increase by $500?" or "What if I save more this month?". While the static CLI command (`logos-cli plan fire`) provides the calculation, it forces users into a tedious process of retyping commands for every 'what-if' scenario. A TUI dashboard unlocks the power of our backend logic by making scenario planning fluid and interactive, bridging the gap between raw compute power and user exploration.

## 📈 Metric Definition
- Success = Users can launch the TUI, navigate to the FIRE dashboard, and adjust input parameters using the keyboard, seeing real-time visual updates.
- Usage Metric = Time spent in the FIRE TUI screen and the number of scenarios simulated per session.

## 🔍 Gap Analysis
Currently, users are restricted to one-shot CLI commands or must export data to spreadsheets for interactive modeling. Spreadsheets disconnect the user from our strict double-entry ledger. By building this into our TUI, we bring the immediate feedback of a spreadsheet into our native ecosystem.

## ✅ Acceptance Criteria
- Must introduce a new dedicated FIRE dashboard screen in the `logos-tui`.
- Must provide interactive input fields for:
  - Monthly Expenses
  - Monthly Savings
  - Liquid Assets
- Must instantly recalculate and display the "FIRE Number", "Safe Net Worth", and "Progress %" upon any input change.
- Must leverage the existing `FireSimulator` and `NetWorthProjector` core logic.

## 🚫 Out of Scope
- Modifying the underlying financial engine or simulation math.
- Automatic live fetching of market data or account balances (manual input only for V1).
- Advanced charts or graphs (Phase 2).
