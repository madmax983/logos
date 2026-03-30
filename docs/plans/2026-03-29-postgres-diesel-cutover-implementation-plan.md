# Postgres + Diesel Cutover Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace AletheiaDB completely with a Postgres + Diesel storage layer while preserving ledger invariants, append-only corrections, and journal as-of reporting.

**Architecture:** Introduce a storage-neutral contract crate, add a Postgres implementation crate with Diesel migrations, port the journal and operational persistence flows, then remove all Aletheia-specific runtime, CLI, tests, and docs. Keep temporal behavior only for journal queries and keep migrations explicit via CLI commands.

**Tech Stack:** Rust 2024, PostgreSQL 16, Diesel 2, diesel_migrations 2, testcontainers + Postgres module, Docker Compose, existing `logos-core` and `logos-proof` invariants.

---

### Task 1: Add a storage-neutral contract crate

**Files:**
- Modify: `Cargo.toml`
- Create: `crates/logos-store/Cargo.toml`
- Create: `crates/logos-store/src/lib.rs`
- Create: `crates/logos-store/src/error.rs`
- Create: `crates/logos-store/src/model.rs`
- Create: `crates/logos-store/src/traits.rs`
- Test: `crates/logos-store/tests/model_smoke.rs`

**Step 1: Write the failing test**

Create `crates/logos-store/tests/model_smoke.rs` with coverage for:

- constructing a neutral stored transaction record
- constructing a fetch run record
- formatting `StoreError`
- trait-object compilation for a `LedgerStore`

**Step 2: Run test to verify it fails**

Run: `cargo test -p logos-store --test model_smoke`

Expected: FAIL because `logos-store` does not exist yet.

**Step 3: Write minimal implementation**

Create the new crate with:

- neutral record types copied from the current storage-facing API surface
- `StoreError`
- `LedgerStore` trait with the methods `logos-runtime` already needs

Keep names close to the current `Stored*` types to reduce churn.

**Step 4: Run test to verify it passes**

Run: `cargo test -p logos-store --test model_smoke`

Expected: PASS

**Step 5: Commit**

```bash
git add Cargo.toml crates/logos-store
git commit -m "feat(store): add neutral ledger store contract crate"
```

### Task 2: Refactor runtime and CLI to use neutral store models

**Files:**
- Modify: `crates/logos-runtime/Cargo.toml`
- Modify: `crates/logos-runtime/src/error/mod.rs`
- Modify: `crates/logos-runtime/src/models/mod.rs`
- Modify: `crates/logos-runtime/src/runtime/mod.rs`
- Modify: `crates/logos-cli/Cargo.toml`
- Modify: `crates/logos-cli/src/commands/analytics.rs`
- Modify: `crates/logos-cli/src/commands/close.rs`
- Modify: `crates/logos-cli/src/commands/fetch.rs`
- Modify: `crates/logos-cli/src/commands/month.rs`
- Modify: `crates/logos-cli/src/commands/reconcile.rs`
- Test: `crates/logos-cli/tests/e2e_happy_path.rs`

**Step 1: Write the failing test**

Add or update an assertion in `crates/logos-cli/tests/e2e_happy_path.rs` that imports neutral models from `logos-store` instead of `logos-store-aletheia`.

**Step 2: Run test to verify it fails**

Run: `cargo test -p logos-cli --test e2e_happy_path e2e_runtime_reopen_restores_persisted_transactions -- --exact`

Expected: FAIL with import or type errors because runtime and CLI still depend on `logos-store-aletheia`.

**Step 3: Write minimal implementation**

Replace all public-facing imports of `logos_store_aletheia::model::*` and `logos_store_aletheia::StoreError` with neutral equivalents from `logos-store`.

Do not change behavior yet. This task only removes storage-type leakage from runtime and CLI.

**Step 4: Run test to verify it passes**

Run:

- `cargo test -p logos-cli --test e2e_happy_path e2e_runtime_reopen_restores_persisted_transactions -- --exact`
- `cargo test -p logos-runtime`

Expected: PASS

**Step 5: Commit**

```bash
git add crates/logos-runtime crates/logos-cli
git commit -m "refactor(runtime): depend on neutral store models"
```

### Task 3: Scaffold the Postgres store crate and migration harness

**Files:**
- Modify: `Cargo.toml`
- Create: `crates/logos-store-pg/Cargo.toml`
- Create: `crates/logos-store-pg/src/lib.rs`
- Create: `crates/logos-store-pg/src/error.rs`
- Create: `crates/logos-store-pg/src/connection.rs`
- Create: `crates/logos-store-pg/src/migrate.rs`
- Create: `crates/logos-store-pg/tests/common/mod.rs`
- Test: `crates/logos-store-pg/tests/db_smoke.rs`
- Create: `docker-compose.yml`

**Step 1: Write the failing test**

Create `crates/logos-store-pg/tests/db_smoke.rs` that:

- boots Postgres with testcontainers
- creates a connection
- runs a simple `SELECT 1`
- calls the migration status helper

**Step 2: Run test to verify it fails**

Run: `cargo test -p logos-store-pg --test db_smoke`

Expected: FAIL because the crate and harness do not exist yet.

**Step 3: Write minimal implementation**

Add the new crate with:

- Diesel Postgres dependencies
- a sync `PgConnection` connection helper
- migration helpers for `pending_migrations` and `run_pending`
- a root-level `docker-compose.yml` that mirrors Autumn's local Postgres ergonomics

Use `DATABASE_URL` as the main contract.

**Step 4: Run test to verify it passes**

Run: `cargo test -p logos-store-pg --test db_smoke`

Expected: PASS

**Step 5: Commit**

```bash
git add Cargo.toml docker-compose.yml crates/logos-store-pg
git commit -m "feat(store-pg): scaffold postgres store crate and migration harness"
```

### Task 4: Add Diesel migrations for the journal core

**Files:**
- Create: `crates/logos-store-pg/migrations/00000000000000_create_journal/up.sql`
- Create: `crates/logos-store-pg/migrations/00000000000000_create_journal/down.sql`
- Create: `crates/logos-store-pg/src/schema.rs`
- Test: `crates/logos-store-pg/tests/journal_schema.rs`

**Step 1: Write the failing test**

Create `crates/logos-store-pg/tests/journal_schema.rs` that:

- runs pending migrations against a disposable database
- verifies the existence of `transactions`, `postings`, and `transaction_corrections`
- verifies unique and foreign-key constraints with intentional bad inserts

**Step 2: Run test to verify it fails**

Run: `cargo test -p logos-store-pg --test journal_schema`

Expected: FAIL because the migration and schema files do not exist.

**Step 3: Write minimal implementation**

Create the initial migration for:

- `transactions`
- `postings`
- `transaction_corrections`

Include indexes for:

- `transactions (effective_at, recorded_at)`
- `postings (transaction_id)`
- `postings (account)`
- `transaction_corrections (supersedes_transaction_id)`

Add `src/schema.rs` matching the migrated tables.

**Step 4: Run test to verify it passes**

Run: `cargo test -p logos-store-pg --test journal_schema`

Expected: PASS

**Step 5: Commit**

```bash
git add crates/logos-store-pg/migrations crates/logos-store-pg/src/schema.rs crates/logos-store-pg/tests/journal_schema.rs
git commit -m "feat(store-pg): add journal schema and diesel migrations"
```

### Task 5: Implement transaction and posting persistence

**Files:**
- Modify: `crates/logos-store-pg/src/lib.rs`
- Create: `crates/logos-store-pg/src/model.rs`
- Create: `crates/logos-store-pg/src/read.rs`
- Create: `crates/logos-store-pg/src/write.rs`
- Test: `crates/logos-store-pg/tests/journal_transactions.rs`

**Step 1: Write the failing test**

Create `crates/logos-store-pg/tests/journal_transactions.rs` covering:

- write balanced transaction
- reopen store and reload transaction
- count transactions
- load postings in stable ordinal order

Port the equivalent happy-path behavior from `crates/logos-store-aletheia/tests/store_contract.rs`.

**Step 2: Run test to verify it fails**

Run: `cargo test -p logos-store-pg --test journal_transactions`

Expected: FAIL because journal persistence is not implemented.

**Step 3: Write minimal implementation**

Implement:

- `write_transaction`
- `write_transaction_with_valid_time`
- `transactions`
- `transaction_count`
- `has_transaction`

Use a database transaction so the transaction row and posting rows commit atomically.

**Step 4: Run test to verify it passes**

Run: `cargo test -p logos-store-pg --test journal_transactions`

Expected: PASS

**Step 5: Commit**

```bash
git add crates/logos-store-pg/src crates/logos-store-pg/tests/journal_transactions.rs
git commit -m "feat(store-pg): persist journal transactions and postings"
```

### Task 6: Implement correction links and journal as-of queries

**Files:**
- Modify: `crates/logos-store-pg/src/read.rs`
- Modify: `crates/logos-store-pg/src/write.rs`
- Test: `crates/logos-store-pg/tests/journal_history.rs`

**Step 1: Write the failing test**

Create `crates/logos-store-pg/tests/journal_history.rs` covering:

- correction rejects unknown superseded transaction ids
- latest correction target is queryable
- `transactions_as_of_us(valid_cutoff, recorded_cutoff)` returns the same visibility semantics as the current Aletheia-backed behavior
- corrected transactions disappear from current-state reads when superseded before the as-of recorded cutoff

**Step 2: Run test to verify it fails**

Run: `cargo test -p logos-store-pg --test journal_history`

Expected: FAIL because correction writes and as-of SQL do not exist.

**Step 3: Write minimal implementation**

Implement:

- `write_correction`
- `latest_correction`
- `transactions_as_of_us`

Use SQL that filters on:

- `transactions.effective_at <= valid_cutoff`
- `transactions.recorded_at <= recorded_cutoff`
- correction rows whose `recorded_at <= recorded_cutoff`

Exclude superseded transactions only when the correction is visible at the requested recorded cutoff.

**Step 4: Run test to verify it passes**

Run: `cargo test -p logos-store-pg --test journal_history`

Expected: PASS

**Step 5: Commit**

```bash
git add crates/logos-store-pg/src crates/logos-store-pg/tests/journal_history.rs
git commit -m "feat(store-pg): add correction links and as-of journal queries"
```

### Task 7: Implement budget and import persistence

**Files:**
- Create: `crates/logos-store-pg/migrations/00000000000001_create_budget_and_import/up.sql`
- Create: `crates/logos-store-pg/migrations/00000000000001_create_budget_and_import/down.sql`
- Modify: `crates/logos-store-pg/src/schema.rs`
- Modify: `crates/logos-store-pg/src/read.rs`
- Modify: `crates/logos-store-pg/src/write.rs`
- Test: `crates/logos-store-pg/tests/budget_and_import.rs`

**Step 1: Write the failing test**

Create `crates/logos-store-pg/tests/budget_and_import.rs` covering:

- writing and reloading month-scoped budget targets
- writing import batches and import records
- deduplicating import records by deterministic content hash
- reopening the store and observing persisted import state

**Step 2: Run test to verify it fails**

Run: `cargo test -p logos-store-pg --test budget_and_import`

Expected: FAIL because the tables and queries do not exist.

**Step 3: Write minimal implementation**

Add migrations and Diesel queries for:

- `budget_targets`
- `import_batches`
- `import_records`

Use a unique constraint for import content hash identity so idempotency is enforced in the database.

**Step 4: Run test to verify it passes**

Run: `cargo test -p logos-store-pg --test budget_and_import`

Expected: PASS

**Step 5: Commit**

```bash
git add crates/logos-store-pg/migrations crates/logos-store-pg/src crates/logos-store-pg/tests/budget_and_import.rs
git commit -m "feat(store-pg): persist budget targets and import history"
```

### Task 8: Implement fetch, reconciliation, month close, and analytics persistence

**Files:**
- Create: `crates/logos-store-pg/migrations/00000000000002_create_ops_tables/up.sql`
- Create: `crates/logos-store-pg/migrations/00000000000002_create_ops_tables/down.sql`
- Modify: `crates/logos-store-pg/src/schema.rs`
- Modify: `crates/logos-store-pg/src/read.rs`
- Modify: `crates/logos-store-pg/src/write.rs`
- Test: `crates/logos-store-pg/tests/ops_records.rs`

**Step 1: Write the failing test**

Create `crates/logos-store-pg/tests/ops_records.rs` covering:

- writing fetch runs
- writing reconciliation runs and linked statement evidence
- writing month closes with reconciliation linkage
- writing analytics artifact manifests including supersession linkage
- reopening and listing all of the above records

Port coverage from the current `logos-store-aletheia` contract tests.

**Step 2: Run test to verify it fails**

Run: `cargo test -p logos-store-pg --test ops_records`

Expected: FAIL because these tables and queries do not exist.

**Step 3: Write minimal implementation**

Add immutable tables and link tables for:

- `statement_lines`
- `fetch_runs`
- `reconciliation_runs`
- `reconciliation_statement_lines`
- `month_closes`
- `analytics_artifacts`

Keep these operational tables immutable with `created_at` timestamps only. Do not add valid-time plus transaction-time columns here.

**Step 4: Run test to verify it passes**

Run: `cargo test -p logos-store-pg --test ops_records`

Expected: PASS

**Step 5: Commit**

```bash
git add crates/logos-store-pg/migrations crates/logos-store-pg/src crates/logos-store-pg/tests/ops_records.rs
git commit -m "feat(store-pg): persist fetch reconciliation close and analytics records"
```

### Task 9: Wire `logos-runtime` to the Postgres store

**Files:**
- Modify: `crates/logos-runtime/Cargo.toml`
- Modify: `crates/logos-runtime/src/runtime/mod.rs`
- Modify: `crates/logos-runtime/src/lib.rs`
- Test: `crates/logos-cli/tests/e2e_happy_path.rs`
- Test: `crates/logos-runtime/tests/havoc_proptest.rs`

**Step 1: Write the failing test**

Update `crates/logos-cli/tests/e2e_happy_path.rs` so the runtime opens a disposable Postgres-backed store rather than the current embedded filesystem store.

Start with one test:

- `e2e_happy_path_posts_and_reports_register_balance`

**Step 2: Run test to verify it fails**

Run: `cargo test -p logos-cli --test e2e_happy_path e2e_happy_path_posts_and_reports_register_balance -- --exact`

Expected: FAIL because runtime still constructs `AletheiaStore`.

**Step 3: Write minimal implementation**

Update `AppRuntime` to:

- open a Postgres-backed store from `DATABASE_URL`
- remove in-memory Aletheia defaults
- preserve existing command behavior

If needed, add a test-only constructor that accepts an already-prepared test database URL.

**Step 4: Run test to verify it passes**

Run:

- `cargo test -p logos-cli --test e2e_happy_path e2e_happy_path_posts_and_reports_register_balance -- --exact`
- `cargo test -p logos-runtime`

Expected: PASS

**Step 5: Commit**

```bash
git add crates/logos-runtime crates/logos-cli/tests/e2e_happy_path.rs crates/logos-runtime/tests
git commit -m "feat(runtime): wire runtime to postgres ledger store"
```

### Task 10: Add explicit database commands and pending-migration guard

**Files:**
- Modify: `crates/logos-cli/src/args.rs`
- Modify: `crates/logos-cli/src/commands/mod.rs`
- Create: `crates/logos-cli/src/commands/db.rs`
- Modify: `crates/logos-cli/src/commands/help.rs`
- Test: `crates/logos-cli/tests/cli_parse.rs`
- Test: `crates/logos-cli/tests/e2e_regressions.rs`

**Step 1: Write the failing test**

Add parse tests for:

- `ledger db status`
- `ledger db migrate`

Add a regression test that runtime startup fails with a clear message when migrations are pending.

**Step 2: Run test to verify it fails**

Run:

- `cargo test -p logos-cli --test cli_parse parses_db_status_command -- --exact`
- `cargo test -p logos-cli --test e2e_regressions`

Expected: FAIL because the `db` command and migration guard do not exist.

**Step 3: Write minimal implementation**

Implement:

- `ledger db status`
- `ledger db migrate`

Use the migration helpers from `logos-store-pg`.

Normal runtime initialization should:

- fail if `DATABASE_URL` is missing
- fail if migrations are pending
- point the user to `ledger db migrate`

**Step 4: Run test to verify it passes**

Run:

- `cargo test -p logos-cli --test cli_parse`
- `cargo test -p logos-cli --test e2e_regressions`

Expected: PASS

**Step 5: Commit**

```bash
git add crates/logos-cli
git commit -m "feat(cli): add explicit db migration commands"
```

### Task 11: Remove Aletheia code, commands, and references

**Files:**
- Modify: `Cargo.toml`
- Delete: `crates/logos-store-aletheia/Cargo.toml`
- Delete: `crates/logos-store-aletheia/src/lib.rs`
- Delete: `crates/logos-store-aletheia/src/model.rs`
- Delete: `crates/logos-store-aletheia/src/read.rs`
- Delete: `crates/logos-store-aletheia/src/write.rs`
- Delete: `crates/logos-store-aletheia/tests/store_contract.rs`
- Delete: `crates/logos-store-aletheia/tests/store_contract_analytics.rs`
- Delete: `crates/logos-store-aletheia/tests/store_contract_correction.rs`
- Delete: `crates/logos-store-aletheia/tests/store_contract_import_batch.rs`
- Delete: `crates/logos-store-aletheia/tests/store_contract_month_close.rs`
- Delete: `crates/logos-store-aletheia/tests/store_contract_read.rs`
- Delete: `crates/logos-store-aletheia/tests/store_contract_read_visibility.rs`
- Delete: `crates/logos-store-aletheia/tests/havoc_kill_switch.rs`
- Delete: `crates/logos-cli/src/commands/aletheia.rs`
- Modify: `crates/logos-cli/src/commands/mod.rs`
- Modify: `crates/logos-cli/src/args.rs`
- Modify: `crates/logos-cli/tests/cli_parse.rs`
- Modify: `README.md`
- Modify: `docs/adr/0001-v0-storage-and-proof-boundary.md`
- Modify: `docs/adr/0002-embedded-aletheia-runtime.md`
- Create: `docs/adr/0004-postgres-diesel-storage-cutover.md`

**Step 1: Write the failing test**

Update CLI parse tests to assert:

- `aletheia` is no longer a known command
- `db` help text is shown instead

**Step 2: Run test to verify it fails**

Run: `cargo test -p logos-cli --test cli_parse rejects_aletheia_command_after_cutover -- --exact`

Expected: FAIL because Aletheia commands still exist.

**Step 3: Write minimal implementation**

Delete the Aletheia crate and CLI command surface.

Update docs to:

- describe Postgres setup
- describe explicit migration commands
- mark the old Aletheia ADRs as superseded
- add a new ADR documenting the SQL-ledger decision

**Step 4: Run test to verify it passes**

Run:

- `cargo test -p logos-cli --test cli_parse`
- `cargo test --workspace`

Expected: PASS

**Step 5: Commit**

```bash
git add Cargo.toml crates/logos-cli README.md docs/adr
git rm -r crates/logos-store-aletheia crates/logos-cli/src/commands/aletheia.rs
git commit -m "refactor(storage): remove aletheia backend and docs"
```

### Task 12: Add CI for the new Postgres-backed world and verify end-to-end

**Files:**
- Create: `.github/workflows/ci.yml`
- Modify: `README.md`
- Test: workspace verification commands from repository root

**Step 1: Write the failing test**

There is no repo CI file today. Create `.github/workflows/ci.yml` with a first failing version if necessary, then validate it locally by matching its commands manually.

**Step 2: Run test to verify it fails**

Run the intended CI commands manually:

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test -p logos-core -p logos-import -p logos-fetch -p logos-reporting`
- `cargo test --workspace`

Expected: one or more commands will fail until CI and docs are aligned with the new Postgres setup.

**Step 3: Write minimal implementation**

Add CI modeled after Autumn's successful shape:

- lint job
- cross-platform fast test job for non-DB crates
- Linux full-workspace DB-backed test job

Document the local developer flow in `README.md`:

- `docker compose up -d`
- export `DATABASE_URL`
- `cargo run -p logos-cli -- db migrate`
- normal command usage

**Step 4: Run test to verify it passes**

Run:

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`

Expected: PASS

**Step 5: Commit**

```bash
git add .github/workflows/ci.yml README.md
git commit -m "ci: add postgres-backed workspace verification"
```

## Final Verification

Run from `C:\Users\markm\logos\.worktrees\feat-postgres-diesel-cutover`:

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `rg -n "aletheia|Aletheia|logos-store-aletheia" .`

Expected:

- all verification commands pass
- the ripgrep command returns only intentionally historical documentation that has been explicitly marked superseded, or no results at all

## Notes

- Do not add a compatibility migration path from Aletheia. This is a clean storage break.
- Do not reintroduce an in-memory fake backend for production paths.
- Keep journal history temporal. Keep the operational tables ordinary and immutable.
- Keep Verus proof scope in `logos-proof` unchanged unless a proven invariant genuinely needs representation updates.
