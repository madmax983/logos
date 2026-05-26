# 🔭 Vantage: Spec for Expense Anomaly Detection

## 👤 User Story
As a budget-conscious user, I want the system to alert me when a month's expenses in a specific envelope significantly deviate from my historical average, so that I can catch accidental subscriptions, billing errors, or creeping overspending early.

## 🤔 So What? (Business Problem)
Currently, users must manually review their budget envelopes and compare them mentally to past months to spot irregularities. This is tedious and error-prone. Creeping expenses (like a utility bill slowly rising or a forgotten subscription) often go unnoticed until they become a major drain. By providing automated anomaly detection, we transition the tool from passive record-keeping to proactive financial health monitoring, delivering immediate, tangible value (saving money).

## 📈 Metric Definition
- **Success:** The system provides a CLI command `logos-cli report anomalies` that accurately flags envelopes with spending that deviates beyond a configurable threshold from their historical baseline.
- **Usage Metric:** Number of anomalies identified and acted upon (e.g., subsequent budget adjustments or transaction corrections).

## 🔍 Gap Analysis
We currently have strict budget envelopes and projection capabilities, but no automated mechanism to highlight abnormal spending patterns. Users rely on manual TUI navigation or visual inspection of monthly reports. Standard banks offer primitive "unusual spending" alerts, but they lack the context of our strict double-entry ledger and custom budget envelopes.

## ✅ Acceptance Criteria
- Must introduce a new CLI command: `logos-cli report anomalies`.
- Must calculate a baseline for each expense account (e.g., a 6-month moving average).
- Must compare current month-to-date spending against this baseline.
- Must output a concise, actionable report of envelopes exceeding the baseline by a specified threshold (default 20%).
- Must allow the user to override the default threshold and historical window via CLI flags.

## 🚫 Out of Scope
- Predictive budgeting based on seasonality or complex machine learning models (V1 uses simple moving averages).
- Auto-correcting or automatically categorizing transactions based on anomalies.
- Real-time alerts via email/SMS (this is a CLI-driven pull model for now).
