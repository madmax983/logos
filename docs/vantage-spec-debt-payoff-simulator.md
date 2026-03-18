# 🔭 Vantage: Spec for Debt Payoff Simulator

👤 **User Story:**
"As a user with outstanding liabilities (e.g., credit cards, personal loans, or student debt), I want to simulate how long it will take to pay off my debts using different monthly payment amounts and interest rates, so that I can create an actionable plan to become debt-free and understand the total interest cost."

✅ **Acceptance Criteria:**
- **Success Metric:** The simulator correctly outputs the total number of months to payoff and the total interest paid for a given principal, annual interest rate, and monthly payment.
- The simulation must include a failsafe termination condition (e.g., `if months > 1200 { break; }`) to prevent infinite loops in edge cases where progress stalls (e.g., when the monthly payment is less than the monthly interest generated).
- If the failsafe is triggered (indicating the debt will never be paid off at the current rate), the simulator must return a specific warning or error rather than a partial or incorrect result.
- The simulator must correctly calculate monthly interest based on the remaining principal.
- The tool must support adding multiple debts to visualize a combined payoff timeline (e.g., snowball or avalanche methods, though basic individual projection is the minimum viable product).

🚫 **Out of Scope:**
- Automatic synchronization with external debt providers or bank accounts.
- Recommending specific debt consolidation loans or external financial products.
- Variable interest rates that change over time (initial support will assume a fixed APR per debt).
