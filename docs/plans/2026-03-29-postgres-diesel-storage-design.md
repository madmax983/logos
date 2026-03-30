# Postgres + Diesel Storage Design

Date: 2026-03-29
Status: Validated

## Goal

Replace AletheiaDB completely with a Postgres-first storage architecture that feels normal to operate, normal to test, and normal to adopt.

The target is a finance CLI/TUI whose ledger truth lives in ordinary SQL tables, with append-only journal semantics, explicit correction links, and "as of" reporting over the journal core only. Everything else should be plain immutable operational data, not full bi-temporal graph storage.

## Decision Summary

1. Remove AletheiaDB entirely from `logos`.
2. Use PostgreSQL 16 plus Diesel as the only production persistence backend.
3. Keep temporal history only where it matters: the journal core.
4. Model corrections as immutable reversing/correcting ledger entries linked to prior transactions.
5. Use explicit migration commands. Do not auto-run schema changes on startup.
6. Mirror Autumn's operator ergonomics where they help:
   - root-level `docker-compose.yml` for local Postgres
   - `DATABASE_URL` as the main connection contract
   - embedded Diesel migrations available to the app and CLI
   - CI split between fast non-DB checks and Linux DB-backed integration tests
7. Treat this as a storage break. No Aletheia data migration is required.

## Why This Replaces Aletheia Cleanly

`logos` is mostly storing ordinary ledger and operations data:

- transactions
- postings
- corrections
- budget targets
- import batches and records
- statement lines
- fetch runs
- reconciliation runs
- month closes
- analytics artifact manifests

The only capability that truly benefits from temporal querying is journal reporting. The current Aletheia-backed `transactions_as_of_us(valid_time, tx_time)` behavior is useful and should survive, but it does not justify a graph-temporal database for the rest of the app.

Most SQL-backed ledgers do not make the entire system fully bi-temporal. They keep the journal immutable, represent mistakes as new correcting or reversing entries, and use timestamps for effective date plus system-recorded date. That is the pattern `logos` should adopt.

## Target Architecture

```mermaid
graph TD
    CLI[logos-cli]
    TUI[logos-tui]
    RT[logos-runtime]
    STORE[logos-store]
    PG[logos-store-pg]
    DB[(PostgreSQL 16)]
    CORE[logos-core]
    IMPORT[logos-import]
    FETCH[logos-fetch]
    REPORT[logos-reporting]
    PROOF[logos-proof]

    CLI --> RT
    TUI --> RT
    RT --> STORE
    RT --> PG
    PG --> STORE
    PG --> DB
    RT --> CORE
    RT --> IMPORT
    RT --> FETCH
    RT --> REPORT
    PROOF --> CORE
```

### Crate Responsibilities

- `logos-core`
  Own domain invariants, transaction balancing, correction rules, and planning logic.
- `logos-store`
  Own storage-neutral record types, store errors, and store traits/contracts.
- `logos-store-pg`
  Own Diesel schema, SQL queries, migration harness, and Postgres persistence.
- `logos-runtime`
  Own orchestration across domain, import, fetch, reporting, and store calls.
- `logos-cli` and `logos-tui`
  Own interface behavior only.
- `logos-proof`
  Remains unchanged as the proof boundary for critical invariants.

## Data Model

### Journal Core

These tables are append-only and support historical reporting:

- `transactions`
- `postings`
- `transaction_corrections`

`transactions` stores immutable transaction facts:

- `id`
- `description`
- `effective_at`
- `recorded_at`
- `source_kind`
- optional source metadata such as import batch id or external reference

`postings` stores the debit and credit legs:

- `transaction_id`
- `ordinal`
- `account`
- `amount_cents`

`transaction_corrections` links a correcting transaction to the superseded transaction:

- `id`
- `supersedes_transaction_id`
- `correcting_transaction_id`
- `reason`
- `recorded_at`

The invariant remains the same:

- every transaction balances exactly
- no transaction row is updated after posting
- corrections create additional facts instead of rewriting prior ones

### Operational Tables

These tables are immutable but not bi-temporal:

- `budget_targets`
- `import_batches`
- `import_records`
- `statement_lines`
- `fetch_runs`
- `reconciliation_runs`
- `reconciliation_statement_lines`
- `month_closes`
- `analytics_artifacts`

These rows only need ordinary creation timestamps and uniqueness constraints. They do not need the full valid-time plus transaction-time model.

## Query Model

### Current-State Reads

Current-state ledger reads filter on:

- all journal transactions recorded so far
- visible transactions not superseded by a correction policy

For many reports, "current state" is simply:

- all transactions with `effective_at <= now`
- all rows recorded so far
- excluding superseded transactions when the business rule says the correcting entry replaces prior intent

### As-Of Reads

`logos` should preserve the useful Aletheia capability through SQL:

`transactions_as_of_us(valid_cutoff, recorded_cutoff)`

This becomes a journal query over:

- `effective_at <= valid_cutoff`
- `recorded_at <= recorded_cutoff`
- correction visibility also bounded by `recorded_at <= recorded_cutoff`

This gives the same core behavior as today without forcing every other table into temporal storage theater.

## Corrections

The correction model is:

- original transaction remains immutable
- correcting transaction is posted as a new immutable transaction
- link row records that the newer transaction supersedes the earlier one
- reason string remains explicit and queryable

This matches common SQL-ledger practice better than mutating historical records. It keeps audit trails obvious and makes reversal behavior explainable to users and auditors without graph semantics.

## Database Access Strategy

Postgres and Diesel are the primary stack. The operator story mirrors Autumn, but the runtime keeps one important simplification:

- explicit migrations like Autumn
- root Docker Compose like Autumn
- `DATABASE_URL` like Autumn
- embedded migrations like Autumn
- but synchronous Diesel connections for the first cutover

Why sync Diesel first:

- `logos` is currently a command-driven CLI/TUI, not a long-lived async web service
- `AppRuntime` is synchronous today
- forcing `diesel-async` through every runtime path would create unnecessary churn during the storage replacement

If `logos` later grows a daemon or API service, `diesel-async` can be introduced with far less risk once the relational schema is stable.

## Operations

### Environment

`DATABASE_URL` is required.

Recommended local default:

`postgres://logos:logos@localhost:5432/logos`

### Local Development

Provide a root-level `docker-compose.yml` with:

- image `postgres:16`
- database `logos`
- user `logos`
- password `logos`
- mapped port `5432`

### Migrations

Migrations are explicit only:

- `ledger db status`
- `ledger db migrate`

Normal command startup should:

- check database reachability
- check for pending migrations
- fail fast with a clear message if migrations are pending

No automatic schema mutation should happen during regular command execution.

## Testing and CI

### Store Tests

All current `logos-store-aletheia` contract tests should be ported to Postgres-backed integration tests.

Use:

- testcontainers for disposable Postgres databases in integration tests
- targeted contract tests for journal persistence and as-of queries
- targeted tests for import, fetch, reconciliation, month close, and analytics persistence

### CI Shape

Mirror Autumn's working CI style:

- lint job for formatting and clippy
- fast cross-platform jobs for crates that do not require Postgres
- Linux job for full DB-backed workspace tests

This avoids pretending Windows and macOS runners should run Docker-heavy integration paths if they do not need to.

## Out of Scope

- any live migration from Aletheia data
- support for multiple persistence backends
- full temporal history on non-journal operational tables
- async pool work for the initial cutover

## Follow-Up Docs

After implementation starts, add:

- a new ADR superseding the Aletheia-first storage decision
- README updates for Postgres setup and migration commands
- removal or supersession notes for Aletheia-specific ADRs and plans
