# 🔭 Vantage: Spec for Income Router Overflow Safety

👤 **User Story:**
"As a user with a large income stream, I want the automated income routing system to handle massive values safely without crashing, so that even highly aggregated or unusual deposits are distributed accurately or explicitly rejected."

🤔 **So What?**
A financial system must gracefully process all valid numerical inputs. When the income router attempts to calculate percentage allocations for very large deposits, it panics due to unguarded arithmetic overflow during multiplication. A hard application crash destroys trust, interrupts the workflow, and leaves the system in an unknown state. Graceful bounding or explicit rejection guarantees a robust, reliable user experience.

🎯 **Metric Definition:**
Success = 0 system panics when processing edge-case large deposits through the Income Router. The system correctly evaluates the routing rules by safely capping values or returning a clear, understandable error when limits are exceeded.

🔍 **Gap Analysis:**
Currently, `IncomeRouter::route_income` computes the split using standard multiplication `(amount_cents * i64::from(rule.percentage)) / 100`, which panics when `amount_cents` approaches `i64::MAX`. Production-grade financial planning systems avoid these crashes by utilizing saturating math operations or explicitly validating that the calculation remains within bounds prior to execution.

✅ **Acceptance Criteria:**
- The `route_income` function must not crash or panic when calculating percentage splits for inputs up to `i64::MAX`.
- The math operations for calculating allocation amounts must be explicitly bounded to prevent integer overflow.
- All existing tests, including `income_router_panics_on_overflow`, must pass without crashing.

🚫 **Out of Scope:**
- Transitioning to an arbitrary-precision decimal type for the entire ledger.
- Providing visual error dialogs in the UI (only robust calculation and error emission are required).
