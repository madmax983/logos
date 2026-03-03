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
- financial planning primitives in `logos-core::planning` (`RsuAutoDistributor`, `FireSimulator`, `NetWorthProjector`)

See [docs/financial-planning.md](docs/financial-planning.md) for details on auto distribution, FIRE progress modeling, and net worth projection.

## Tooling

Run from repository root:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --offline -- -D warnings
cargo test --workspace --offline
```

Verus proofs:

```sh
/path/to/verus/verus logos-proof/transaction_invariants.verus
/path/to/verus/verus logos-proof/correction_invariants.verus
/path/to/verus/verus logos-proof/budget_invariants.verus
/path/to/verus/verus logos-proof/rsu_policy_invariants.verus
```

## Example Commands

```sh
# Optional: override local embedded DB path (default is ~/.logos/ledger or %USERPROFILE%\.logos\ledger)
export LOGOS_DB_PATH="/path/to/logos/.data/ledger"

cargo run -p logos-cli -- txn add --description "paycheck" --debit-account "assets:checking" --credit-account "income:salary" --amount-cents 100000
cargo run -p logos-cli -- budget set --month 2026-03 --budget-cents 300000 --expense-account-prefix "expenses:"
cargo run -p logos-cli -- report month --month 2026-03 --checking-account "assets:checking"
cargo run -p logos-tui
```

`budget set` persists month-scoped targets in the embedded store. `report month` is month-windowed using transaction effective time.

## Embedded Persistence + History

- Storage is local and in-process by default; no HTTP server is required for normal CLI usage.
- Writes are append-only journal entities with correction links in `logos-store-aletheia`.
- Historical reads are available via history APIs (`valid_time`, `tx_time`) such as:
  - `AletheiaStore::transactions_as_of(valid_time, tx_time)`

### Backup / Restore Basics

- Stop writer processes before taking a filesystem backup of your ledger directory.
- Backup the directory configured by `LOGOS_DB_PATH` (or the default profile path).
- Restore by replacing that directory and starting the CLI again.

## Run Local AletheiaDB

Use the CLI helper to launch and check a real local `aletheia-server` instance when you want HTTP integration/testing. This is optional for local CLI persistence.

```sh
# Optional if your checkout is not at /path/to/gallifreydb
export ALETHEIADB_MANIFEST_PATH="/path/to/gallifreydb/Cargo.toml"

# Start server (foreground)
cargo run -p logos-cli -- aletheia start

# In a second terminal, check health
cargo run -p logos-cli -- aletheia status
```

Health checks use:

- `GALLIFREYDB_HOST` (default: `127.0.0.1`, with `0.0.0.0` normalized to localhost)
- `GALLIFREYDB_PORT` (default: `8080`)
