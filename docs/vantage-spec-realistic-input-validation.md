# 🔭 Vantage: Spec for Realistic Input Validation

👤 **User Story:**
"As a user running cashflow and RSU projections, I want the system to reject ridiculously large inputs upfront, so that I get a helpful validation error instead of crashing the system with an overflow panic."

🤔 **So What?**
Havoc chaos engineering tests revealed that projecting massive inputs (like `i64::MAX`) causes arithmetic overflow panics in `CashflowProjector` (`current_balances.entry(...).or_insert(0) += posting.amount()`) and `project_rsu_forecast_summary` (`.sum::<i64>()`). Refactoring the entire core to use BigInt or saturating math everywhere introduces unnecessary complexity and performance overhead. Instead, enforcing a realistic cap (e.g., max $1 Billion) at the CLI and boundary level provides immediate user feedback and protects the system from crashes, keeping the core logic fast and simple. Complexity is a cost.

🎯 **Metric Definition:**
Success = 0 panics during Havoc testing for large inputs in projections, returning graceful validation error messages instead.

🔍 **Gap Analysis:**
The current `logos-core` logic relies on standard `i64` integer math and unchecked aggregations (like `+=` in `project_balances` and `.sum::<i64>()` in RSU forecasting). It assumes users will not input values large enough to overflow 64-bit integers. However, missing boundary layer validation allows these massive inputs to reach and crash the core engines.

✅ **Acceptance Criteria:**
- The CLI must validate all incoming monetary values (e.g. amounts, expenses, vest units) against a defined realistic maximum (e.g., 100 billion dollars).
- Any input exceeding this bound must be rejected with an `InputExceedsRealisticBounds` error.
- The boundary layer must catch these errors before calling `project_balances` or `project_rsu_forecast_summary`.

🚫 **Out of Scope:**
- Modifying the core domain math inside `CashflowProjector` or `logos-reporting` to use BigInteger or widespread saturating operations.
