# 🔭 Vantage: Spec for FIRE CLI Commands

## 👤 User Story
As a planner aiming for Financial Independence, Retire Early (FIRE),
I want to be able to project my progress toward my FIRE number using CLI commands,
so that I can understand how my current net worth, burn rate, and unvested RSUs contribute to my goal.

## 🤔 So What? (Business Problem)
Users currently lack a way to leverage the existing `FireSimulator` and `NetWorthProjector` domain logic from `logos-core` directly in the CLI. Providing a first-class CLI experience unlocks the value of these core planning primitives, shifting them from theoretical libraries to actionable financial tools that answer the question "When can I stop working?".

## 📈 Metric Definition
- Success = Users can run `logos-cli plan fire` and receive an immediate projection summary.
- Usage Metric = Number of FIRE projections run locally.

## 🔍 Gap Analysis
Existing tools like Personal Capital or generic spreadsheets are either privacy-invasive, inflexible, or lack integration with our strict double-entry ledger. Our core library has the capability, but we lack the 'Human Interface'.

## ✅ Acceptance Criteria
- A new `logos-cli` subcommand under `plan fire` (or similar) must exist to invoke the `FireSimulator`.
- The command must accept inputs for `monthly-expenses` and `safe-withdrawal-rate-pct`.
- The command must output the computed FIRE number, safe net worth, and progress percentage.
- (Optional) A `plan project` command should simulate future net worth milestones.
- The output format should be human-readable, ideally leveraging standard UI tables/formatting for clarity.
- All amounts must be formatted clearly (e.g., dollars/cents).

## 🚫 Out of Scope
- Integration with external broker APIs for live RSU/stock prices (must use manual or pre-fetched values).
- TUI integration (this spec is strictly for the CLI subcommands).
- Modifying the underlying `FireSimulator` domain logic or risk haircut tiers.
