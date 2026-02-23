# logos

Forward-looking personal finance CLI/TUI with strict double-entry, budget envelopes, RSU planning policy, and `logos-proof` Verus invariants.

## Current Status

Implemented foundations:

- workspace crate boundaries (`logos-core`, `logos-cli`, `logos-import`, `logos-store-aletheia`, `logos-reporting`, `logos-tui`)
- strict transaction balancing and correction semantics in `logos-core`
- budget rollover and RSU policy invariants in `logos-core`
- deterministic CSV fingerprint dedupe in `logos-import`
- in-memory Aletheia adapter contract in `logos-store-aletheia`
- core projections in `logos-reporting`
- read-only TUI app/view skeleton in `logos-tui`
- Verus spine proofs in `logos-proof`

## Tooling

Run from repository root:

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --offline -- -D warnings
cargo test --workspace --offline
```

Verus proofs:

```powershell
C:\Users\markm\verus\verus.exe logos-proof\transaction_invariants.verus
C:\Users\markm\verus\verus.exe logos-proof\correction_invariants.verus
C:\Users\markm\verus\verus.exe logos-proof\budget_invariants.verus
C:\Users\markm\verus\verus.exe logos-proof\rsu_policy_invariants.verus
```

## Example Commands (Current Skeleton)

```powershell
cargo run -p logos-cli -- txn add --description "paycheck"
cargo run -p logos-tui
```

`logos-tui` is currently read-only skeleton output; rich terminal widgets come in later tasks.
