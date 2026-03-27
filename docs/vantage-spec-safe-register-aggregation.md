# 🔭 Vantage: Spec for Safe Register Aggregation

## 👤 **User Story:**
As a high-net-worth user or system operator aggregating large transaction histories, I want my financial reports to calculate accurately without crashing or silently corrupting data, so that I can trust the integrity of my ledger balances.

## 🤔 **So What?**
What business problem does this solve?
The reporting engine currently panics or silently caps values when calculating extremely large register balances (arithmetic overflow). While 64-bit integer limits (`i64::MAX`) seem unreachable for standard dollars, aggregating in cents over decades of transactions or dealing with hyper-inflated currencies can hit these bounds. Data corruption (via silent saturation) or abrupt crashes destroy user trust in a financial system. Strict accuracy and transparent error handling are absolute requirements for an accounting tool.

## 🎯 **Metric Definition:**
- **Success:** 0 panics during register balance projection, even when inputs exceed `i64::MAX`.
- **Success:** 0 instances of silent data corruption (e.g., capping at maximum limits via saturating math).

## 🔍 **Gap Analysis:**
Currently, the system experiences arithmetic overflows when summing large register entries. Previous attempts to fix this might use `.saturating_add()`, which prevents the panic but silently caps the financial balance at `i64::MAX`. Looking at professional accounting systems, they never silently alter data; they either use arbitrarily large number types (like `BigInt` or `BigDecimal`) or fail explicitly with a clear error.

## ✅ **Acceptance Criteria:**
- The register balance aggregation must safely handle operations that would exceed the 64-bit integer limit.
- The system must never use `.saturating_add()` to handle monetary aggregations.
- If an overflow occurs, the system must explicitly handle it by returning an error (`Result`) or by safely using wider data types (e.g., `BigInt`).

## 🚫 **Out of Scope:**
- Rewriting the underlying `AletheiaDB` storage engine to use `BigInt` natively.
- Modifying the core transaction schema.
