# 🔭 Vantage: Spec for RSU Budget Plan Overflow Safety

👤 **User Story:**
"As a planner managing my RSU equity budget, I want the system to safely handle extremely large financial commitments without crashing, so that a massive input or compounding edge-case doesn't break my entire budget projection view."

🤔 **So What?**
A financial reporting engine must be resilient to all inputs, including extreme outliers. When the system panics on large inputs (like `i64::MIN`), it entirely breaks the user's ability to view their budget plans. A crash destroys trust in the platform's stability. Gracefully handling extreme numbers by returning an explicit bounds error or capping them ensures the software remains usable under all conditions.

🎯 **Metric Definition:**
Success = 0 panics during RSU budget plan projections, even when negative variables or edge-cases (e.g. `i64::MIN` commitments) trigger overflow situations. The system must fail gracefully or handle limits safely.

🔍 **Gap Analysis:**
Currently, `project_rsu_budget_plan` panics when presented with extreme negative variables like `fixed_commitments_cents` at `i64::MIN` because it attempts arithmetic operations (likely subtraction) that exceed standard integer limits. Robust financial tools use saturating math, checked arithmetic, or validate/cap bounds early to prevent system crashes on edge case inputs.

✅ **Acceptance Criteria:**
- The `project_rsu_budget_plan` function must not panic when processing extreme bounds (such as `i64::MIN` for commitments).
- The arithmetic logic within the budget planner must use safe checked or saturating operations when calculating residuals or subtracting values.
- All existing tests, including `rsu_budget_havoc`, must pass successfully after the fix without expecting a panic.

🚫 **Out of Scope:**
- Transitioning the core storage to use arbitrary-precision numbers.
- Adding complex UI error dialogs for overflow states.
