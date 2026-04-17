# REQUIRES FEATURE NOVA

# logos

Forward-looking personal finance CLI/TUI with strict double-entry, budget envelopes, RSU planning policy, and `logos-proof` Verus invariants.

## Current Status

Implemented foundations:

- workspace crate boundaries (`logos-core`, `logos-cli`, `logos-import`, `logos-store`, `logos-store-pg`, `logos-reporting`, `logos-tui`)
- strict transaction balancing and correction semantics in `logos-core`
- budget rollover and RSU policy invariants in `logos-core`
- deterministic CSV fingerprint dedupe in `logos-import`
- storage-neutral store contract in `logos-store`
- Postgres + Diesel persistence adapter in `logos-store-pg`
- core projections in `logos-reporting`
- read-only TUI app/view skeleton in `logos-tui`
- Verus spine proofs in `logos-proof` (transactions, corrections, budgets, RSU policy, import dedupe/idempotence)
- financial planning primitives in `logos-core::planning` (`RsuAutoDistributor`, `FireSimulator`, `NetWorthProjector`)

See [docs/financial-planning.md](docs/financial-planning.md) for details on auto distribution, FIRE progress modeling, and net worth projection.

Headless statement-fetch architecture for future `month autopilot` automation is tracked in [docs/adr/0003-headless-statement-fetch-and-autopilot-integration.md](docs/adr/0003-headless-statement-fetch-and-autopilot-integration.md). Operator guidance for config layout, 1Password refs, fetch-run triage, and scheduled execution lives in [docs/fetch-ops.md](docs/fetch-ops.md).

## Tooling

Run from repository root:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Verus proofs:

```sh
/path/to/verus/verus logos-proof/transaction_invariants.verus
/path/to/verus/verus logos-proof/correction_invariants.verus
/path/to/verus/verus logos-proof/budget_invariants.verus
/path/to/verus/verus logos-proof/rsu_policy_invariants.verus
/path/to/verus/verus logos-proof/import_invariants.verus
```

## Example Commands

```sh
# Required: point logos at Postgres
export DATABASE_URL="postgres://logos:logos@127.0.0.1:5432/logos"

# Optional local convenience database
docker compose up -d db
sleep 3

# Apply explicit migrations
cargo run -p logos-cli -- db migrate

cargo run -p logos-cli -- txn add --description "paycheck" --debit-account "assets:checking" --credit-account "income:salary" --amount-cents 100000
cargo run -p logos-cli -- budget set --month 2026-03 --budget-cents 300000 --expense-account-prefix "expenses:"
cargo run -p logos-cli -- report month --month 2026-03 --checking-account "assets:checking"
cargo run -p logos-tui
```

`budget set` persists month-scoped targets in Postgres. `report month` is month-windowed using transaction effective time.

## Postgres Persistence + History

- Postgres is the primary and only production storage backend.
- Journal writes are append-only with correction links.
- Historical reads use ledger `effective_at` plus `recorded_at` timestamps rather than full-database temporal storage.
- Runtime startup fails fast when pending migrations exist; run `ledger db migrate` explicitly.

### Backup / Restore Basics

- Use normal Postgres backup/restore practice (`pg_dump`, physical backups, managed snapshots, or equivalent).
- Back up artifact files separately when using local parquet analytics output.
