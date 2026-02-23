# logos-proof

This module contains Verus specifications and proof spines for Logos core invariants.

Current proof files:

- transaction balance validity
- correction lineage integrity
- budget rollover conservation
- RSU policy table and allocation invariants

Files:

- `transaction_invariants.verus`
- `correction_invariants.verus`
- `budget_invariants.verus`
- `rsu_policy_invariants.verus`

Run proofs:

```powershell
C:\Users\markm\verus\verus.exe logos-proof\transaction_invariants.verus
C:\Users\markm\verus\verus.exe logos-proof\correction_invariants.verus
C:\Users\markm\verus\verus.exe logos-proof\budget_invariants.verus
C:\Users\markm\verus\verus.exe logos-proof\rsu_policy_invariants.verus
```
