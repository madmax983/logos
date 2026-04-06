# 🔭 Vantage: Spec for Automated Cashflow Recurrence Detection

## 👤 User Story
As a Budgeter, I want the system to automatically identify my recurring income and expenses from past transactions, so that I don't have to manually configure templates for my monthly cashflow projections.

## 🤔 So What? (Business Problem)
Manual budget forecasting is tedious and error-prone. Users often forget recurring subscriptions, utility bills, or irregular income streams. By automatically surfacing these patterns, we reduce the cognitive load of month-over-month planning, increase trust in our forecasting tools, and improve user retention because the software feels "smart" and proactive.

## 📈 Metric Definition
- **Success:** >80% of actual recurring transactions (occurring 3+ times in the last 6 months) are automatically detected and suggested to the user.
- **Performance:** Detection scan over 10,000 historical transactions takes < 500ms.

## 🔍 Gap Analysis
- **Current State:** Users must manually enter all planned cashflows, which is a high-friction setup process.
- **Competitors:** Standard personal finance tools have recurring transaction detection, but they are often inaccurate or don't explicitly feed into a predictive cashflow engine.
- **Our Edge:** Deterministic identification feeding directly into our zero-trust double-entry system.

## ✅ Acceptance Criteria
- The system must analyze historical transactions to identify patterns matching the exact same description, credit account, debit account, and amount.
- A pattern must occur a configurable minimum number of times (e.g., 3) to be flagged as recurring.
- The system must output a set of recurring templates (amount, source, destination, frequency).
- The detection must not alter any existing historical data.

## 🚫 Out of Scope
- Variable amount recurrence detection (e.g., utility bills that change month-to-month).
- Auto-creating future transactions without user confirmation (Phase 2).
- ML-based fuzzy matching.
