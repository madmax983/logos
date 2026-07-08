## 2026-04-12 - Havoc Discovered Debt Optimizer Overflow
**Confusion:** The `DebtOptimizer` panics on arithmetic overflow when calculating interest for massive debt balances because it uses an unguarded multiplication (`balance_cents * interest_rate_pct`).
**Clarification:** As Havoc, I wrote a `#[should_panic]` test to exploit this fragility and demonstrate the system's weakness without fixing it.
## 2024-05-18 - Math Overflow panics in Proptest

**The Trigger:** Input math bounded by percentages is prone to integer overflow if `vest` values are unconstrained (e.g. `i64::MAX`).
**The Crash:** `attempt to multiply with overflow` panics.
**Action:** Havoc doesn't fix bugs, but proving the bounds missing through Proptests keeps the team on their toes.
## Overflow Boundaries & Unbalanced Transactions
When using saturating arithmetic to prevent overflows at extreme boundaries (like `i64::MAX`) in double-entry transaction builders (e.g., portfolio rebalancers), remainder sweeps may fail due to precision loss during percentage divisions. Update chaos tests to accept `DomainError::UnbalancedTransaction` as a valid safe boundary behavior rather than expecting an unconditional success or a panic.

**👺 Havoc: `RunwaySimulator` Panics on Arithmetic Overflow**
🧨 **The Trigger:** Passing `f64::INFINITY` (or large enough `annual_inflation_pct` leading to infinity) to `RunwaySimulator` causes an arithmetic overflow when casting float to integer or multiplying `current_burn`.
📉 **The Stack Trace:** attempt to multiply with overflow / float to int cast panic.
🧪 **Reproduction:** Run `cargo test -p logos-core havoc_runway_simulator_panics_on_overflow --features nova`.
😈 **Comment:** You assumed people wouldn't simulate runway with infinite inflation. You were wrong.
