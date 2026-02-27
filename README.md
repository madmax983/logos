# logos

Forward-looking personal finance CLI/TUI with strict double-entry, budget envelopes, RSU planning policy, and `logos-proof` Verus invariants.

## Current Status

Implemented foundations:

- workspace crate boundaries (`logos-core`, `logos-cli`, `logos-import`, `logos-store-aletheia`, `logos-reporting`, `logos-tui`)
- strict transaction balancing and correction semantics in `logos-core`
- budget rollover and RSU policy invariants in `logos-core`
- deterministic CSV fingerprint dedupe in `logos-import`
- embedded durable Aletheia adapter contract in `logos-store-aletheia`
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
# Optional: override local embedded DB path (default is ~/.logos/ledger or %USERPROFILE%\.logos\ledger)
$env:LOGOS_DB_PATH = "C:\Users\markm\logos\.data\ledger"

cargo run -p logos-cli -- txn add --description "paycheck" --debit-account "assets:checking" --credit-account "income:salary" --amount-cents 100000
cargo run -p logos-tui
```

`logos-tui` is currently read-only skeleton output; rich terminal widgets come in later tasks.

## Run Local AletheiaDB

Use the CLI helper to launch and check a real local `aletheia-server` instance:

```powershell
# Optional if your checkout is not at C:\Users\markm\gallifreydb
$env:ALETHEIADB_MANIFEST_PATH = "C:\Users\markm\gallifreydb\Cargo.toml"

# Start server (foreground)
cargo run -p logos-cli -- aletheia start

# In a second terminal, check health
cargo run -p logos-cli -- aletheia status
```

Health checks use:

- `GALLIFREYDB_HOST` (default: `127.0.0.1`, with `0.0.0.0` normalized to localhost)
- `GALLIFREYDB_PORT` (default: `8080`)
