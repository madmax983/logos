# 🔭 Vantage: Spec for FIRE Progress TUI

## 👤 User Story
As a user planning for early retirement, I want an interactive, visual dashboard in the terminal (TUI) to track my FIRE progress, so that I can see how different withdrawal rates and expenses dynamically affect my timeline without re-running CLI commands.

## 🤔 So What? (Business Problem)
The CLI implementation of FIRE projection requires users to iteratively re-run commands with different arguments to explore scenarios. This friction prevents deep engagement with the financial planning logic. An interactive TUI bridges this gap by providing real-time feedback and visual progress, increasing product stickiness and user delight.

## 📈 Metric Definition
- Success: Users can navigate to a dedicated "FIRE Dashboard" within the TUI.
- Engagement: Time spent in the FIRE TUI view > 2 minutes per session.

## 🔍 Gap Analysis
Currently, the TUI only provides a read-only ledger/transaction view. Tools like Personal Capital have rich, interactive retirement planners, but they are web-based and not privacy-first. We have the underlying simulation engine, but lack a local, visually interactive frontend.

## ✅ Acceptance Criteria
- Must add a new tab/view in the TUI for "FIRE Progress".
- Must display current net worth, FIRE number, and a visual progress bar.
- Must allow adjusting the monthly expense assumption interactively via keyboard inputs (e.g., up/down arrows to change expenses).
- Must recalculate and update the projected FIRE timeline and progress bar in real-time as assumptions change.

## 🚫 Out of Scope
- Integration with live broker data (relies on manual input or existing store data).
- Modifying the core domain logic.
- Complex Monte Carlo charts in the terminal (Phase 3).
