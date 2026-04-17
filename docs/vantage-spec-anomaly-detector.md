# 🔭 Vantage: Spec for Anomaly Detector

👤 **User Story:**
"As a user reviewing my monthly expenses, I want to automatically identify unusually large transactions, so that I can catch potential fraud, mistakes, or overspending without manually scanning every row."

🤔 **So What?**
Users often miss fraudulent charges or mistaken duplicate charges when buried in hundreds of regular transactions. Automating outlier detection saves time and prevents financial loss by highlighting actionable items requiring human review.

🎯 **Metric Definition:**
Success = 95% of transactions that are mathematical outliers are correctly flagged for review, while keeping false positives below 5% of total transactions.

🔍 **Gap Analysis:**
Currently, users must manually read their ledger or rely on basic visual reports. Standard banking apps offer "large purchase" alerts, but they are often fixed amounts rather than dynamically relative to the user's typical spending patterns for specific accounts or categories.

✅ **Acceptance Criteria:**
- The system must dynamically calculate the normal spending range based on historical data.
- The system must flag any new transaction that falls significantly outside this normal range.
- The system must provide a clear description of the flagged transaction and why it was flagged.
- The user must be able to adjust the sensitivity of the detection.

🚫 **Out of Scope:**
- Automatic dispute resolution with banks.
- Real-time blocking of transactions.
- Complex machine learning models.
