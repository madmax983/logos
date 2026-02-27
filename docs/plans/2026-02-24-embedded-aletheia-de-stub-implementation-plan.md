# Embedded Aletheia De-Stub Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace placeholder/in-memory CLI behavior with a real embedded persistent ledger path backed by `AletheiaDB`, including historical (bi-temporal) query support.

**Architecture:** Keep `logos-core` as the invariant boundary and use `logos-store-aletheia` as the only persistence adapter. Wire `logos-cli` commands to runtime/store (not placeholder handlers), and make persistence local/in-process (no HTTP requirement for normal CLI usage). Model writes as append-only journal entities and correction edges, then layer as-of reads over valid time + transaction time.

**Tech Stack:** Rust 2024 workspace, `logos-core`, `logos-cli`, `logos-store-aletheia`, local `aletheiadb` crate (embedded), Verus proof artifacts in `logos-proof`.

---

### Task 1: De-Stub `txn add` Command Execution Path

**Files:**
- Modify: `crates/logos-cli/src/commands/txn.rs`
- Modify: `crates/logos-cli/src/args.rs`
- Test: `crates/logos-cli/src/commands/txn.rs` (unit tests in-module)

**Step 1: Write failing tests**

Add unit tests for:
- empty description rejection
- successful runtime posting path (via fake writer)
- runtime error propagation through `CliError`

**Step 2: Run tests to verify fail**

Run: `cargo test -p logos-cli commands::txn --offline`  
Expected: FAIL with missing helper/trait path.

**Step 3: Implement minimal runtime-backed command**

Implement:
- runtime writer trait in `txn.rs`
- helper that calls runtime posting API
- top-level `add` function that prints created transaction id
- add `CliError` variant for command runtime failures

**Step 4: Run tests to verify pass**

Run: `cargo test -p logos-cli commands::txn --offline`  
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/logos-cli/src/commands/txn.rs crates/logos-cli/src/args.rs
git commit -m "feat(cli): wire txn add command to runtime write path"
```

### Task 2: Add Explicit `txn add` Economic Inputs (No Hidden Defaults)

**Files:**
- Modify: `crates/logos-cli/src/args.rs`
- Modify: `crates/logos-cli/src/commands/txn.rs`
- Modify: `crates/logos-cli/tests/cli_parse.rs`

**Step 1: Write failing parser tests**

Require flags:
- `--description`
- `--debit-account`
- `--credit-account`
- `--amount-cents`

Add tests for missing/invalid values.

**Step 2: Run tests to verify fail**

Run: `cargo test -p logos-cli --test cli_parse --offline`  
Expected: FAIL.

**Step 3: Implement parser + command payload**

Update `TxnCommand::Add` payload and dispatch signature. Parse `amount_cents` as `i64` with explicit error.

**Step 4: Run tests to verify pass**

Run: `cargo test -p logos-cli --test cli_parse --offline`  
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/logos-cli/src/args.rs crates/logos-cli/src/commands/txn.rs crates/logos-cli/tests/cli_parse.rs
git commit -m "feat(cli): require explicit txn add debit credit and amount flags"
```

### Task 3: Introduce Embedded Store Interface Boundary

**Files:**
- Modify: `crates/logos-store-aletheia/src/lib.rs`
- Modify: `crates/logos-store-aletheia/src/write.rs`
- Modify: `crates/logos-store-aletheia/src/read.rs`
- Test: `crates/logos-store-aletheia/tests/store_contract.rs`

**Step 1: Write failing contract tests**

Define tests for:
- open store at path
- write persists across reopen
- correction chain survives reopen

**Step 2: Run tests to verify fail**

Run: `cargo test -p logos-store-aletheia --test store_contract --offline`  
Expected: FAIL.

**Step 3: Implement store opening + persistence contract API**

Introduce constructor(s):
- `open(path: &Path)` for durable mode
- `new_in_memory()` for tests/fallback

Maintain append-only semantics.

**Step 4: Run tests to verify pass**

Run: `cargo test -p logos-store-aletheia --test store_contract --offline`  
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/logos-store-aletheia/src crates/logos-store-aletheia/tests/store_contract.rs
git commit -m "feat(store): add durable open/reopen contract for append-only ledger data"
```

### Task 4: Wire `logos-store-aletheia` to Embedded `AletheiaDB`

**Files:**
- Modify: `crates/logos-store-aletheia/Cargo.toml`
- Modify: `crates/logos-store-aletheia/src/lib.rs`
- Modify: `crates/logos-store-aletheia/src/model.rs`
- Modify: `crates/logos-store-aletheia/src/write.rs`
- Modify: `crates/logos-store-aletheia/src/read.rs`

**Step 1: Write failing integration tests for embedded adapter**

Test entity/edge mapping:
- transaction node + posting nodes/edges written
- correction edge written
- query round-trip reconstructs domain transaction

**Step 2: Run tests to verify fail**

Run: `cargo test -p logos-store-aletheia --test store_contract --offline`  
Expected: FAIL.

**Step 3: Implement minimal mapping**

Mapping model:
- `LedgerTransaction`
- `LedgerPosting`
- `LedgerCorrection`
- stable properties (`txn_id`, `description`, `amount_cents`, `account`, timestamps)

**Step 4: Run tests to verify pass**

Run: `cargo test -p logos-store-aletheia --test store_contract --offline`  
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/logos-store-aletheia/Cargo.toml crates/logos-store-aletheia/src
git commit -m "feat(store): embed AletheiaDB adapter for transaction and correction persistence"
```

### Task 5: CLI Runtime Uses Durable Embedded Store by Default

**Files:**
- Modify: `crates/logos-cli/src/runtime.rs`
- Modify: `crates/logos-cli/src/lib.rs`
- Modify: `crates/logos-cli/src/main.rs`
- Test: `crates/logos-cli/tests/e2e_happy_path.rs`

**Step 1: Write failing runtime durability test**

Create test:
- run write path
- drop runtime
- reopen runtime
- verify transaction exists and register projection reflects persisted data

**Step 2: Run test to verify fail**

Run: `cargo test -p logos-cli --test e2e_happy_path --offline`  
Expected: FAIL.

**Step 3: Implement durable runtime bootstrap**

Add runtime constructor that loads store path from env:
- `LOGOS_DB_PATH` (default local file path in user profile/workdir)

**Step 4: Run tests to verify pass**

Run: `cargo test -p logos-cli --test e2e_happy_path --offline`  
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/logos-cli/src/runtime.rs crates/logos-cli/src/lib.rs crates/logos-cli/src/main.rs crates/logos-cli/tests/e2e_happy_path.rs
git commit -m "feat(cli): default runtime to durable embedded store path"
```

### Task 6: Add Bi-Temporal Fields and As-Of Read API

**Files:**
- Modify: `crates/logos-store-aletheia/src/model.rs`
- Modify: `crates/logos-store-aletheia/src/read.rs`
- Modify: `crates/logos-reporting/src/register.rs`
- Test: `crates/logos-store-aletheia/tests/store_contract.rs`

**Step 1: Write failing as-of tests**

Cases:
- backdated `valid_time` write visible in historical read
- newer correction hidden in earlier `tx_time` snapshot

**Step 2: Run tests to verify fail**

Run: `cargo test -p logos-store-aletheia --test store_contract --offline`  
Expected: FAIL.

**Step 3: Implement as-of read methods**

Add API:
- `transactions_as_of(valid_time, tx_time)`
- correction-aware reconstruction

**Step 4: Run tests to verify pass**

Run: `cargo test -p logos-store-aletheia --test store_contract --offline`  
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/logos-store-aletheia/src crates/logos-reporting/src/register.rs crates/logos-store-aletheia/tests/store_contract.rs
git commit -m "feat(store): add bi-temporal as-of reads for historical ledger views"
```

### Task 7: Replace Remaining Placeholder Command Handlers

**Files:**
- Modify: `crates/logos-cli/src/commands/budget.rs`
- Modify: `crates/logos-cli/src/commands/report.rs`
- Modify: `crates/logos-cli/src/runtime.rs`
- Test: `crates/logos-cli/tests/e2e_happy_path.rs`

**Step 1: Write failing tests for budget/report commands**

Ensure command handlers call runtime/reporting layer and produce deterministic output.

**Step 2: Run tests to verify fail**

Run: `cargo test -p logos-cli --offline`  
Expected: FAIL.

**Step 3: Implement command wiring**

Remove placeholder prints and wire to runtime/report projections.

**Step 4: Run tests to verify pass**

Run: `cargo test -p logos-cli --offline`  
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/logos-cli/src/commands crates/logos-cli/src/runtime.rs crates/logos-cli/tests
git commit -m "feat(cli): replace budget/report placeholders with real runtime-backed handlers"
```

### Task 8: Proof + Regression Alignment

**Files:**
- Modify: `logos-proof/transaction_invariants.verus`
- Modify: `logos-proof/correction_invariants.verus`
- Modify: `logos-proof/README.md`
- Test: `crates/logos-core/tests/*.rs`

**Step 1: Write failing regression tests for any newly uncovered edge behavior**

Cover persistence + correction chain + as-of behavior assumptions in runtime-facing tests.

**Step 2: Run tests/proofs to verify fail where expected**

Run:
- `cargo test -p logos-core --offline`
- `C:\Users\markm\verus\verus.exe logos-proof\transaction_invariants.verus`
- `C:\Users\markm\verus\verus.exe logos-proof\correction_invariants.verus`

**Step 3: Update proofs/specs and core invariants**

Bring formal invariants in line with persisted historical behavior.

**Step 4: Run tests/proofs to verify pass**

Run same commands; expect PASS.

**Step 5: Commit**

```bash
git add logos-proof crates/logos-core/tests
git commit -m "proof: align transaction and correction invariants with durable bi-temporal persistence"
```

### Task 9: Docs + Operator Runbook Cleanup

**Files:**
- Modify: `README.md`
- Create: `docs/adr/0002-embedded-aletheia-runtime.md`
- Modify: `docs/adr/0001-v0-storage-and-proof-boundary.md`

**Step 1: Write failing docs checklist**

Checklist:
- no “placeholder” language in active CLI flows
- embedded mode is default
- HTTP server listed as optional integration mode only

**Step 2: Update docs**

Document:
- local DB path
- backup/recovery basics
- as-of query concepts

**Step 3: Verify commands in docs**

Run each documented command once and confirm behavior.

**Step 4: Commit**

```bash
git add README.md docs/adr/0002-embedded-aletheia-runtime.md docs/adr/0001-v0-storage-and-proof-boundary.md
git commit -m "docs: publish embedded runtime runbook and update storage ADRs"
```

### Task 10: Final Verification Gate

**Files:**
- Verify workspace + proofs + CLI flows (no code changes required unless fixes found)

**Step 1: Run full quality gates**

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --offline -- -D warnings
cargo test --workspace --offline
C:\Users\markm\verus\verus.exe logos-proof\transaction_invariants.verus
C:\Users\markm\verus\verus.exe logos-proof\correction_invariants.verus
C:\Users\markm\verus\verus.exe logos-proof\budget_invariants.verus
C:\Users\markm\verus\verus.exe logos-proof\rsu_policy_invariants.verus
```

**Step 2: Run manual smoke flows**

- `cargo run -p logos-cli -- txn add ...`
- `cargo run -p logos-cli -- report month`
- restart process and confirm persisted reads

**Step 3: Commit any final fixes**

```bash
git add -A
git commit -m "chore: final verification pass for embedded durable ledger runtime"
```
