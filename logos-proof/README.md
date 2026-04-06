# logos-proof

This module contains Verus specifications and proof spines for Logos core invariants.

Current proof files:

- transaction balance validity and bi-temporal snapshot visibility
- correction lineage integrity and tx-time supersede visibility behavior
- budget rollover conservation and latest-write-wins target semantics
- RSU policy table and allocation invariants
- import dedupe partition and idempotent re-import behavior

Files:

- `transaction_invariants.verus`
- `correction_invariants.verus`
- `budget_invariants.verus`
- `rsu_policy_invariants.verus`
- `import_invariants.verus`

Run proofs:

```sh
/path/to/verus/verus logos-proof/transaction_invariants.verus
/path/to/verus/verus logos-proof/correction_invariants.verus
/path/to/verus/verus logos-proof/budget_invariants.verus
/path/to/verus/verus logos-proof/rsu_policy_invariants.verus
/path/to/verus/verus logos-proof/import_invariants.verus
```
