# 🔭 Vantage: Spec for Tax Loss Harvester

## 👤 User Story
As an investor holding taxable accounts, I want the system to analyze my asset holdings and identify unrealized losses, so that I can strategically sell them to offset capital gains and reduce my tax liability.

## 🤔 So What? (Business Problem)
Tax loss harvesting is a powerful strategy, but manually tracking cost bases and current market values across multiple assets is tedious and error-prone. By highlighting potential harvesting opportunities, the software provides direct financial ROI to the user.

## 📈 Metric Definition
Success = The system correctly identifies assets whose current market value is below their calculated cost basis and quantifies the potential tax deduction.

## 🔍 Gap Analysis
Our current portfolio tools track balances and allocations, but do not actively flag assets for tax optimization strategies.

## ✅ Acceptance Criteria
- Must ingest current asset prices and compare them against the recorded cost basis of lots in the ledger.
- Must identify lots with unrealized losses.
- Must sum the total potential harvestable loss.
- Must output a report listing the specific assets, lots, and the magnitude of the potential loss.

## 🚫 Out of Scope
- Actually executing trades on a brokerage platform.
- Calculating exact tax bill reductions based on the user's specific income tax bracket.
- Handling wash sale rule violations (Phase 2).
