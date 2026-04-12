## 2026-04-12 - Havoc Discovered Debt Optimizer Overflow
**Confusion:** The `DebtOptimizer` panics on arithmetic overflow when calculating interest for massive debt balances because it uses an unguarded multiplication (`balance_cents * interest_rate_pct`).
**Clarification:** As Havoc, I wrote a `#[should_panic]` test to exploit this fragility and demonstrate the system's weakness without fixing it.
