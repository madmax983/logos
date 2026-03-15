# 🔭 Vantage: Spec for Planning CLI Commands

👤 **User Story:**
"As a software engineer planning my financial future, I want to use the `logos` CLI to run FIRE simulations, project my net worth, and distribute my RSU vests according to a policy, so that I can see the impact of my savings and equity compensation without leaving my terminal."

✅ **Acceptance Criteria:**
- **RSU Distribute Command:**
  - Given an `rsu distribute` command with gross vest amount and policy percentages, it must output a preview of the resulting multi-leg transaction (tax, goals, smoothing, discretionary).
  - Must validate that the policy percentages sum to 100%.
- **FIRE Simulator Command:**
  - Given a `fire simulate` command with monthly expenses, current assets/liabilities, and upcoming vests, it must output the calculated FIRE number, safe net worth, and progress percentage.
  - Must default to a 4% safe withdrawal rate if not specified.
- **Net Worth Projector Command:**
  - Given a `net-worth project` command with initial net worth, monthly savings, upcoming vests, milestones, and a month horizon (e.g., 60 months), it must output a chronological table of projected net worth and indicate when milestones are crossed.
- **Usability:** All commands must clearly display monetary values in dollars/cents (e.g., `$1,250.00`) despite taking raw cent inputs or using internal cent representations.

🚫 **Out of Scope:**
- Automatically persisting the RSU distribution transaction to the ledger (Phase 1 is preview only).
- Storing FIRE or Net Worth configurations/state in the Aletheia store (these run purely in-memory based on CLI arguments for now).
- Complex CLI interactive prompts (TUI integration is separate).
- Advanced tax withholding calculations (the distributor simply takes a flat percentage).
