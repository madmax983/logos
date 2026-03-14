# 🔭 Vantage: Spec for Discrete Financial Shocks

👤 **User Story:**
"As a Financial Planner, I want to model discrete, one-time future expenses (such as a house downpayment, a wedding, or a sabbatical) so that I can see how large cash outflows impact my FIRE date and Net Worth trajectory over time."

✅ **Acceptance Criteria:**
- **Success Metric:** The Net Worth Projector must accurately deduct the value of the shock in the specific month it occurs, adjusting all subsequent monthly balances accordingly.
- A user can define an "Expense Shock" with a target date (e.g., Month 36) and an exact monetary amount.
- The timeline output must clearly identify the month in which an "Expense Shock" took place, alongside its value, so the user understands the sudden drop in net worth.
- Any milestones (like reaching a FIRE number) that were previously crossed must not be retroactively "uncrossed," but the new, lower trajectory must correctly delay future milestones.
- The projection must still balance to the penny, failing fast if an expense shock drives the projected net worth below zero without a valid debt instrument.

🚫 **Out of Scope:**
- Complex amortized loans or mortgages (this feature models the initial cash outflow only).
- Variable market returns on the remaining balance (projections continue to use simple linear accumulation).
- Automatic calendar synchronization for upcoming life events.
