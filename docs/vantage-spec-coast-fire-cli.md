# 🔭 Vantage: Spec for Coast FIRE CLI

## 👤 User Story
As an investor aiming for financial freedom, I want to calculate my Coast FIRE milestone via the CLI, so that I know exactly how much I need invested today to stop saving and let compound interest cover my retirement.

## 🤔 So What? (Business Problem)
Users are often demoralized by massive, multi-million dollar FIRE numbers. Coast FIRE breaks this down into a far more achievable near-term milestone (e.g., reaching $300k invested in your 30s). The core Coast FIRE simulation logic already exists in our backend library but is completely inaccessible to our end-users. Exposing this via the CLI transforms a hidden calculation into a high-value product feature that answers a critical user question: "When can I stop actively saving?"

## 📈 Metric Definition
- **Success:** Users can run a CLI command (e.g., `analytics coast-fire`) and receive an immediate projection summary.
- **Usage Metric:** Number of Coast FIRE projections run locally.

## 🔍 Gap Analysis
Current financial tools often only focus on the final retirement number, ignoring intermediate milestones like Coast FIRE that offer users flexibility (e.g., switching to a lower-paying, lower-stress job). We have the domain logic built, but lack the 'Human Interface'.

## ✅ Acceptance Criteria
- Must expose a `coast-fire` command in the CLI.
- Must accept user inputs for `monthly-expenses`, `annual-growth-rate-pct`, `years-to-retirement`, and current assets.
- Must output a clear, readable summary that includes the Target FIRE Number, the Required Coast FIRE Number, and a status indicator of whether the user is currently "coasting".
- Must use existing core simulation logic.
- Output should be neatly formatted in a human-readable table (e.g., using standard UI tables/formatting).

## 🚫 Out of Scope
- Interactive TUI screens for Coast FIRE (Phase 2).
- Automatic pulling of asset balances from the ledger (V1 will require manual input).
- Integration with external broker APIs for live stock prices.
