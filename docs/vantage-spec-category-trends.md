# 🔭 Vantage: Spec for Category Trend Analyzer

## 👤 User Story
As a user managing a budget, I want to see my spending aggregated by high-level category groups over time, so that I can identify trends and adjust my financial behavior.

## 🤔 So What? (Business Problem)
Raw transaction data is too granular for macro-level financial planning. Users need to zoom out and see aggregated trends (e.g., "Housing" vs "Transportation" vs "Food"). An automated category trend analyzer provides this high-level view without requiring manual spreadsheet manipulation.

## 📈 Metric Definition
Success = The system can accurately map transaction accounts to predefined category groups and output the aggregated spending per group.

## 🔍 Gap Analysis
The system currently records exact double-entry postings, but lacks a built-in mechanism to roll up these detailed accounts into broader, actionable category trends.

## ✅ Acceptance Criteria
- Must provide a mapping between specific accounts and high-level category groups.
- Must aggregate transaction postings according to this mapping.
- Must accurately handle refunds/credits within a category (netting out the total).
- Must output the total spending per category group over a defined set of transactions.

## 🚫 Out of Scope
- Automatic machine learning classification of uncategorized transactions.
- Highly detailed sub-category nesting (focus is on top-level groups).
