## 2026-04-12 - Havoc Discovered Debt Optimizer Overflow
**Confusion:** The `DebtOptimizer` panics on arithmetic overflow when calculating interest for massive debt balances because it uses an unguarded multiplication (`balance_cents * interest_rate_pct`).
**Clarification:** As Havoc, I wrote a `#[should_panic]` test to exploit this fragility and demonstrate the system's weakness without fixing it.
## 2024-05-18 - Math Overflow panics in Proptest

**The Trigger:** Input math bounded by percentages is prone to integer overflow if `vest` values are unconstrained (e.g. `i64::MAX`).
**The Crash:** `attempt to multiply with overflow` panics.
**Action:** Havoc doesn't fix bugs, but proving the bounds missing through Proptests keeps the team on their toes.
