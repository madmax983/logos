# 🔭 Vantage: Spec for Emergency Runway Calculator

## 👤 User Story
"As a cautious saver, I want to calculate how many months of living expenses I have saved, so that I can understand my financial resilience during a loss of income."

## 💼 So What? (Business Problem)
Many people know their bank balances but struggle to translate that into time. Knowing you have "$15,000" is less actionable than knowing you have "3.5 months of baseline expenses". Providing a time-based metric reduces financial anxiety and helps users set clear emergency fund goals. Complexity is a cost; utility is a revenue. We need to turn raw numbers into understandable survival metrics.

## 📏 Metric Definition
- **Success:** The calculator correctly outputs a "runway in months" metric with at least one decimal point of precision.
- **Performance:** Calculation executes in under 50ms to ensure the CLI remains perfectly responsive.

## 🔍 Gap Analysis
Current market tools either require complex Excel modeling or only show total assets without tying them to historical spending rates. Standard libraries don't solve this out-of-the-box. By using the user's defined budget envelopes (baseline expenses), we can automatically provide a personalized runway calculation without manual input, something generic budgeting apps often fail to integrate directly with ledger data.

## ✅ Acceptance Criteria
- Must calculate runway by dividing liquid savings by average monthly baseline expenses.
- Must allow the user to optionally exclude certain accounts (e.g., restricted RSUs or retirement accounts) from the "liquid savings" pool.
- Must fall back to a user-provided "assumed monthly expense" if no historical budget data exists.
- Must gracefully handle the case where expenses are 0 (must not crash due to division by zero panics).

## 🚫 Out of Scope
- Projecting inflation adjustments over the runway period.
- Calculating runway for discrete financial shocks (e.g., medical emergency + job loss simultaneously).
- Automated tax penalty calculations for withdrawing from restricted accounts.
