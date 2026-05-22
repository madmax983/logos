# 🔭 Vantage: Spec for Multi-Currency Support

## 👤 User Story
As an international professional, I want to record and report my finances in multiple currencies (e.g., USD, EUR) so that I can accurately track my global net worth without manual conversions outside the system.

## 💼 So What? (Business Problem)
Currently, Logos only supports a single currency natively (hardcoded USD formatting). In a globalized world, users often hold assets or incur liabilities in different currencies. Lacking multi-currency means these users must either maintain separate databases or do manual, error-prone FX conversions prior to input, breaking the strict "double-entry" guarantees since exchange rates fluctuate. Adding multi-currency support expands our total addressable market to expats, international remote workers, and crypto holders.

## 🎯 Metric Definition
- **Success =** 100% of generated financial reports automatically balance in a user-defined base currency.
- **Success =** Zero manual FX conversions required by the user at transaction entry time.

## 📊 Gap Analysis / Acceptance Criteria
- **Must handle different commodities:** Transactions must be able to specify a currency ticker (e.g., USD, EUR, BTC) along with the amount.
- **Must support conversion rates:** The system needs a way to define or fetch exchange rates to convert foreign currencies to the base currency for aggregate reporting (like Net Worth).
- **Must preserve exact foreign amounts:** The ledger must store the exact amount in the original currency, not just the converted amount, to prevent rounding drift and historical inaccuracy.
- **Must support custom display formatting:** Each currency should be formatted according to its conventions (e.g., `$1,500.00` vs `1.500,00 €`).

## 🚫 Out of Scope
- Real-time fetching of FX rates (Phase 1 will rely on user-provided static rate tables).
- Complex capital gains tracking purely for currency fluctuation.