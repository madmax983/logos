# Vantage Spec: Multi-Currency Support

## 👤 User Story
As an international professional or frequent traveler, I want to record transactions and track accounts in multiple currencies, so that I can accurately reflect my global net worth without manual external conversions.

## 💼 Business Problem
Currently, the system assumes a single base currency (e.g., USD in integer cents). Users holding foreign bank accounts or taking international trips are forced to manually calculate exchange rates at the time of transaction. This leads to inaccurate tracking over time and a poor user experience.

## 🎯 Success Metrics
- 100% of multi-currency transactions balance correctly in their native currency.
- Net Worth Projector can aggregate and display a unified base-currency net worth.
- Zero arithmetic panics due to conversion precision issues.

## ✅ Acceptance Criteria
- Must support specifying a currency or commodity for any account.
- Must support transactions where postings involve mixed currencies, utilizing an exchange rate or explicit cost basis.
- Must fail gracefully and return a structured error if a transaction is unbalanced within each respective currency or lacks an explicit conversion rate.
- Must retain exact precision for base amounts, using defined minimum divisible units.

## 🚫 Out of Scope
- Real-time automated fetching of live FX rates (Phase 1 will rely on static rate configurations).
- Cryptocurrency decimals support.