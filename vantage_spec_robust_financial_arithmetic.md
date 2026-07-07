👤 **User Story:**
As a user performing financial projections, I want the system to safely handle extreme inputs (like massive salaries, huge expenses, or extremely long time horizons) so that the application returns a clear error instead of crashing.

🤔 **So What? (Business Problem):**
Financial simulation tools are prone to edge cases where extreme user inputs (e.g. typing too many zeros) cause system-wide crashes. A crashing application damages user trust and appears unpolished. Gracefully handling these extremes by providing clear validation errors ensures a reliable, professional user experience.

📈 **Metric Definition:**
- Success = 0% crash rate when extreme values are input during financial planning commands.
- All extreme input cases return a user-friendly error message.

🔍 **Gap Analysis:**
Currently, our projection modules assume reasonable constraints on user inputs (like time horizons or monetary amounts). When these assumptions are violated by extreme numbers, the application terminates abruptly. Standard financial tools validate bounds and return clear errors before performing calculations.

✅ **Acceptance Criteria:**
- The system must gracefully handle massive monetary values for income, expenses, portfolios, and RSU vests without crashing.
- The system must gracefully handle extremely long projection horizons without crashing.
- When an input exceeds safe calculation limits, the system must return a clear, user-friendly error message indicating that the value is too large.
- No calculations should result in a hard application crash or panic.

🚫 **Out of Scope:**
- Automatically capping or modifying user inputs to fit within limits (users must explicitly correct their inputs).
- Changing the core financial formulas for normal inputs.
