👤 **User Story:**
As a user running long-term financial simulations, I want the system to gracefully handle extremely large numbers and long time horizons so that the application remains stable and provides clear error messages instead of abruptly crashing.

🤔 **So What? (Business Problem):**
Our financial planning and simulation tools currently crash when users input massive amounts (like unusually large salaries, enormous stock grants, or extremely long retirement horizons). This creates a fragile user experience and reduces trust. By safely handling these extreme bounds, we ensure enterprise-grade reliability and prevent unexpected system failures.

📈 **Metric Definition:**
- Success = Zero system crashes when users input extreme financial values or simulation lengths.
- Quality Metric = All extreme boundary errors result in a clear, human-readable error message or safely capped limits.

🔍 **Gap Analysis:**
The current simulation engine assumes reasonable user inputs and lacks protective boundaries for extreme cases. Standard financial tools either use arbitrary limits or cap inputs, whereas our system attempts to process them and fails. We need to introduce sensible limits and safe calculations across all simulation logic.

✅ **Acceptance Criteria:**
- Must safely process extremely large financial inputs (e.g., massive account balances, income, or stock grants) without system crashes.
- Must reject or safely cap extremely long time horizons (e.g., forecasting centuries into the future) with a friendly warning.
- Must prevent system resource exhaustion when running high-volume simulations by enforcing a maximum number of simulation paths.
- Must ensure that percentage-based allocations and rebalancing logic do not fail when applied to exceptionally large totals.

🚫 **Out of Scope:**
- Upgrading the underlying system to support infinitely large numbers.
