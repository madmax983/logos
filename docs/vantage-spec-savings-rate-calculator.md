# 🔭 Vantage: Spec for Savings Rate Calculator

## 👤 User Story
As a user tracking my finances, I want to calculate my monthly and annual savings rate, so that I can measure my progress toward financial independence and ensure I am retaining a healthy portion of my income.

## 🤔 So What? (Business Problem)
Knowing absolute savings is useful, but the savings rate (percentage of income saved) is the true leading indicator of time-to-retirement. Without this feature, users have to manually export their income and expenses to calculate this vital metric. Providing this automatically increases the product's value as a holistic financial planning tool rather than just a ledger.

## 📈 Metric Definition
Success = The CLI can output a clear, calculated savings rate percentage (e.g., "35.5%") based on income and expense transactions for a given period.

## 🔍 Gap Analysis
Currently, `logos-cli` provides budget envelopes and month-windowed reports, but does not synthesize income vs. total expenses into a single top-level savings rate percentage. We need to bridge this gap to align with FIRE (Financial Independence, Retire Early) principles.

## ✅ Acceptance Criteria
- Must calculate savings rate as `(Total Income - Total Expenses) / Total Income`.
- Must allow querying the savings rate for a specific month or year.
- Must display the result as a percentage with up to two decimal places.
- Must gracefully handle periods with zero income.

## 🚫 Out of Scope
- Granular categorization of "types" of savings (e.g., pre-tax vs. post-tax savings rates) for the initial version.
- Complex tax-advantaged account tracking.
