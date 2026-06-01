# 🔭 Vantage: Spec for Coast FIRE CLI

## 👤 User Story
As a user planning for financial independence, I want to calculate my Coast FIRE number via the CLI, so that I can easily determine if I have enough invested today to stop saving and coast to my retirement goal.

## 🤔 So What? (Business Problem)
The backend `CoastFireSimulator` logic exists, but is not accessible to non-technical users. By exposing this in the CLI, we unlock a key lifestyle planning tool that lets users know when they can confidently downshift their careers without jeopardizing their retirement.

## 📈 Metric Definition
- Success = Users can run `logos-cli plan coast-fire` (or similar) and receive their Coast FIRE milestone status.
- Usage Metric = Number of Coast FIRE projections run locally.

## 🔍 Gap Analysis
Users currently have to build their own spreadsheets to calculate the present value of their FIRE target. Bringing this into the CLI makes our tool a one-stop-shop for intermediate financial milestones, differentiating us from basic budgeting apps.

## ✅ Acceptance Criteria
- Must expose a `plan coast-fire` command in the CLI.
- Must accept inputs for the FIRE target (or monthly expenses to derive it), expected annual real growth rate, and years to retirement.
- Must output the Coast FIRE number and whether the user is currently "coasting" (has reached the milestone).
- Must output results in a human-readable format.

## 🚫 Out of Scope
- Dynamic fetching of market growth rates (user must supply the assumed rate).
- TUI integration (Phase 2).
