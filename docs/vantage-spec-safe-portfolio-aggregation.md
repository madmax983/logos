# 🔭 Vantage: Spec for Safe Portfolio Aggregation

👤 **User Story:**
As a High Net Worth Individual (HNWI) or institutional user, I want to aggregate large portfolio balances without the system crashing, so that I can accurately track wealth that exceeds standard system limits.

🤔 **So What?**
A financial application that crashes on large numbers is fundamentally untrustworthy. Handling large numbers safely prevents catastrophic system failures during reporting for our most valuable target demographic and maintains data integrity and user trust.

🎯 **Metric Definition:**
Success = The reporting engine successfully handles aggregate balances exceeding `i64::MAX` with 0 system panics, either by graceful capping or explicit error messaging.

🔎 **Gap Analysis:**
Currently, `project_register_balance_iter` crashes (panics) on arithmetic overflow because it assumes register entries will not exceed the 64-bit integer limit. Standard financial software handles this gracefully via saturating math or arbitrary-precision types to avoid unexpected runtime crashes.

✅ **Acceptance Criteria:**
- The system must handle portfolio additions and aggregations that exceed standard limits without panicking.
- The user must receive a clear error message or saturated value instead of a system crash.

🚫 **Out of Scope:**
- Upgrading the entire underlying storage engine to arbitrary precision integers.
- Implementation details such as specific structs or data types used to fix the overflow.
