# Logos Ledger v0 Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build v0 of a personal finance CLI/TUI tool with strict double-entry posting, budget envelopes with rollover, RSU planning policy, Aletheia-backed persistence, and read-only TUI reporting.

**Architecture:** Use a Rust workspace with domain-first boundaries. `logos-core` owns invariants and command validation, `logos-store-aletheia` owns persistence mapping, `logos-import` owns CSV ingestion + idempotency, `logos-reporting` owns read models, `logos-cli` handles all writes, `logos-tui` is read-only, and `logos-proof` contains Verus specs/proofs for critical invariants.

**Tech Stack:** Rust 2024, clap, serde, thiserror, anyhow, chrono, rust_decimal, ratatui/crossterm, proptest, rstest (optional), Verus (`C:\Users\markm\verus\verus.exe`), Aletheia client crate/integration.

---

### Task 1: Workspace Scaffold and Lint Baseline

**Files:**
- Create: `Cargo.toml`
- Create: `crates/logos-core/Cargo.toml`
- Create: `crates/logos-core/src/lib.rs`
- Create: `crates/logos-cli/Cargo.toml`
- Create: `crates/logos-cli/src/main.rs`
- Create: `crates/logos-store-aletheia/Cargo.toml`
- Create: `crates/logos-store-aletheia/src/lib.rs`
- Create: `crates/logos-import/Cargo.toml`
- Create: `crates/logos-import/src/lib.rs`
- Create: `crates/logos-reporting/Cargo.toml`
- Create: `crates/logos-reporting/src/lib.rs`
- Create: `crates/logos-tui/Cargo.toml`
- Create: `crates/logos-tui/src/main.rs`
- Create: `logos-proof/README.md`
- Create: `logos-proof/transaction_invariants.verus`
- Modify: `.gitignore`

**Step 1: Write the failing workspace smoke test**

Create `crates/logos-core/tests/workspace_smoke.rs`:

```rust
#[test]
fn workspace_smoke() {
    assert_eq!(2 + 2, 4);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p logos-core --test workspace_smoke`  
Expected: FAIL with package/crate not found.

**Step 3: Write minimal workspace implementation**

- Convert repo root into workspace with members under `crates/*`.
- Set `edition = "2024"` and workspace lint tables (rust + clippy).
- Add minimal compileable crates with tiny functions/main stubs.
- Add `logos-proof` folder and starter Verus file placeholder.

Minimal `crates/logos-core/src/lib.rs`:

```rust
pub fn crate_ready() -> bool {
    true
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p logos-core --test workspace_smoke`  
Expected: PASS.

**Step 5: Commit**

```bash
git add Cargo.toml crates logos-proof .gitignore
git commit -m "build: scaffold logos v0 workspace and crate boundaries"
```

### Task 2: Domain Types for Accounting and Budget Taxonomy

**Files:**
- Create: `crates/logos-core/src/domain/mod.rs`
- Create: `crates/logos-core/src/domain/account.rs`
- Create: `crates/logos-core/src/domain/category.rs`
- Modify: `crates/logos-core/src/lib.rs`
- Test: `crates/logos-core/tests/domain_types.rs`

**Step 1: Write the failing test**

`crates/logos-core/tests/domain_types.rs`:

```rust
use logos_core::{AccountType, Category, CategoryGroup};

#[test]
fn account_types_and_category_hierarchy_are_constructible() {
    let group = CategoryGroup::new("needs").unwrap();
    let category = Category::new(group.id().clone(), "rent").unwrap();
    assert_eq!(category.group_id(), group.id());
    assert_eq!(AccountType::Asset.normal_balance_sign(), 1);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p logos-core --test domain_types`  
Expected: FAIL with unresolved imports/types.

**Step 3: Write minimal implementation**

Implement:
- `AccountType` enum with canonical 5 types.
- `normal_balance_sign()` helper.
- `CategoryGroup` and `Category` newtypes with validation.
- Public exports in `lib.rs`.

**Step 4: Run test to verify it passes**

Run: `cargo test -p logos-core --test domain_types`  
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/logos-core/src crates/logos-core/tests/domain_types.rs
git commit -m "feat(core): add canonical account and category domain types"
```

### Task 3: Double-Entry Transaction Validation Core

**Files:**
- Create: `crates/logos-core/src/domain/transaction.rs`
- Create: `crates/logos-core/src/error.rs`
- Modify: `crates/logos-core/src/domain/mod.rs`
- Modify: `crates/logos-core/src/lib.rs`
- Test: `crates/logos-core/tests/transaction_balance.rs`

**Step 1: Write failing tests**

`crates/logos-core/tests/transaction_balance.rs`:

```rust
use logos_core::{Posting, TransactionBuilder};

#[test]
fn balanced_transaction_is_accepted() {
    let txn = TransactionBuilder::new("paycheck")
        .posting(Posting::debit("assets:checking", 100_00))
        .posting(Posting::credit("income:salary", 100_00))
        .build()
        .expect("balanced");
    assert_eq!(txn.postings().len(), 2);
}

#[test]
fn unbalanced_transaction_is_rejected() {
    let err = TransactionBuilder::new("bad")
        .posting(Posting::debit("assets:checking", 100_00))
        .posting(Posting::credit("income:salary", 90_00))
        .build()
        .expect_err("must fail");
    assert!(err.to_string().contains("balanced"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p logos-core --test transaction_balance`  
Expected: FAIL (missing transaction types).

**Step 3: Write minimal implementation**

Add:
- `Posting` type and constructors.
- `TransactionBuilder` with `build()` enforcing sum == 0.
- Domain error enum via `thiserror`.

**Step 4: Run test to verify it passes**

Run: `cargo test -p logos-core --test transaction_balance`  
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/logos-core/src crates/logos-core/tests/transaction_balance.rs
git commit -m "feat(core): enforce strict double-entry write-time validation"
```

### Task 4: Correction Semantics (Append-Only)

**Files:**
- Create: `crates/logos-core/src/domain/correction.rs`
- Modify: `crates/logos-core/src/domain/mod.rs`
- Modify: `crates/logos-core/src/lib.rs`
- Test: `crates/logos-core/tests/corrections.rs`

**Step 1: Write failing tests**

```rust
use logos_core::{Correction, TransactionId};

#[test]
fn correction_links_to_prior_transaction() {
    let original = TransactionId::new("txn-1");
    let correction = Correction::new(original.clone(), "fix memo").unwrap();
    assert_eq!(correction.supersedes_id(), &original);
}

#[test]
fn correction_cannot_supersede_itself() {
    let id = TransactionId::new("txn-1");
    let err = Correction::new(id.clone(), "self").and_then(|c| c.validate_not_self(&id));
    assert!(err.is_err());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p logos-core --test corrections`  
Expected: FAIL.

**Step 3: Write minimal implementation**

Create correction linkage type with basic local cycle guard and public API used by CLI/store layers.

**Step 4: Run test to verify it passes**

Run: `cargo test -p logos-core --test corrections`  
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/logos-core/src crates/logos-core/tests/corrections.rs
git commit -m "feat(core): add append-only correction domain semantics"
```

### Task 5: Budget Model with Monthly Rollover Conservation

**Files:**
- Create: `crates/logos-core/src/domain/budget.rs`
- Modify: `crates/logos-core/src/domain/mod.rs`
- Modify: `crates/logos-core/src/lib.rs`
- Test: `crates/logos-core/tests/budget_rollover.rs`
- Test: `crates/logos-core/tests/budget_prop.rs`

**Step 1: Write failing tests**

- `budget_rollover.rs`: exact formula checks.
- `budget_prop.rs`: property tests for conservation.

Example:

```rust
#[test]
fn rollover_conserves_value() {
    let end = logos_core::rollover_end_balance(1_000_00, 500_00, 1_200_00);
    assert_eq!(end, 300_00);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p logos-core --test budget_rollover`  
Expected: FAIL.

**Step 3: Write minimal implementation**

Implement monthly budget structs and rollover function:

`start + assigned - spent = end`.

**Step 4: Run tests to verify pass**

Run:
- `cargo test -p logos-core --test budget_rollover`
- `cargo test -p logos-core --test budget_prop`

Expected: PASS.

**Step 5: Commit**

```bash
git add crates/logos-core/src crates/logos-core/tests/budget_*.rs
git commit -m "feat(core): add budget envelopes and rollover conservation"
```

### Task 6: RSU Policy and Forecast Domain

**Files:**
- Create: `crates/logos-core/src/domain/rsu.rs`
- Modify: `crates/logos-core/src/domain/mod.rs`
- Modify: `crates/logos-core/src/lib.rs`
- Test: `crates/logos-core/tests/rsu_policy.rs`

**Step 1: Write failing tests**

Test:
- conservative tier selection by horizon (`<30`, `30-90`, `>90`)
- allocation percentages must sum to 100
- forecast object cannot mutate ledger values (domain boundary check)

**Step 2: Run test to verify it fails**

Run: `cargo test -p logos-core --test rsu_policy`  
Expected: FAIL.

**Step 3: Write minimal implementation**

Implement:
- `HaircutTierTable` validation.
- `AllocationPolicy` validation.
- `forecast_value()` using 30-day average input and tier haircut.

**Step 4: Run test to verify it passes**

Run: `cargo test -p logos-core --test rsu_policy`  
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/logos-core/src crates/logos-core/tests/rsu_policy.rs
git commit -m "feat(core): implement rsu haircut and allocation policy invariants"
```

### Task 7: CLI Skeleton (Verb-First) and Error Contracts

**Files:**
- Create: `crates/logos-cli/src/args.rs`
- Create: `crates/logos-cli/src/commands/mod.rs`
- Create: `crates/logos-cli/src/commands/txn.rs`
- Create: `crates/logos-cli/src/commands/budget.rs`
- Create: `crates/logos-cli/src/commands/report.rs`
- Modify: `crates/logos-cli/src/main.rs`
- Test: `crates/logos-cli/tests/cli_parse.rs`

**Step 1: Write failing parser tests**

Test verb-first parsing:

```rust
#[test]
fn parses_txn_add_command() {
    let args = vec!["ledger", "txn", "add", "--description", "paycheck"];
    let parsed = logos_cli::parse_args(args).unwrap();
    assert_eq!(parsed.command_path(), "txn.add");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p logos-cli --test cli_parse`  
Expected: FAIL.

**Step 3: Write minimal implementation**

Use `clap` derive for subcommands and wire to placeholder handlers returning `anyhow::Result<()>`.

**Step 4: Run test to verify it passes**

Run: `cargo test -p logos-cli --test cli_parse`  
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/logos-cli/src crates/logos-cli/tests/cli_parse.rs
git commit -m "feat(cli): add verb-first command skeleton and parser tests"
```

### Task 8: CSV Import Mapping + Deterministic Fingerprint Idempotency

**Files:**
- Create: `crates/logos-import/src/csv.rs`
- Create: `crates/logos-import/src/fingerprint.rs`
- Modify: `crates/logos-import/src/lib.rs`
- Test: `crates/logos-import/tests/fingerprint_idempotency.rs`
- Test: `crates/logos-import/tests/csv_mapping.rs`

**Step 1: Write failing tests**

Test repeated import same row produces same fingerprint and dedupe decision.

**Step 2: Run test to verify it fails**

Run: `cargo test -p logos-import --test fingerprint_idempotency`  
Expected: FAIL.

**Step 3: Write minimal implementation**

Implement deterministic fingerprint function over canonicalized fields:
- source id/path
- timestamp (normalized)
- amount
- memo
- mapped account/category

**Step 4: Run tests to verify pass**

Run:
- `cargo test -p logos-import --test fingerprint_idempotency`
- `cargo test -p logos-import --test csv_mapping`

Expected: PASS.

**Step 5: Commit**

```bash
git add crates/logos-import/src crates/logos-import/tests
git commit -m "feat(import): add csv mapping and deterministic dedupe fingerprints"
```

### Task 9: Aletheia Store Adapter Contract

**Files:**
- Create: `crates/logos-store-aletheia/src/model.rs`
- Create: `crates/logos-store-aletheia/src/write.rs`
- Create: `crates/logos-store-aletheia/src/read.rs`
- Modify: `crates/logos-store-aletheia/src/lib.rs`
- Test: `crates/logos-store-aletheia/tests/store_contract.rs`

**Step 1: Write failing adapter contract tests**

Focus:
- write balanced txn succeeds
- unbalanced txn rejected before persistence
- correction append links to superseded transaction id

**Step 2: Run test to verify it fails**

Run: `cargo test -p logos-store-aletheia --test store_contract`  
Expected: FAIL.

**Step 3: Write minimal implementation**

Introduce:
- store traits (or trait impl) for command/query operations
- conversion layer between domain structs and Aletheia data model
- placeholder/mock-backed implementation first if live daemon not wired

**Step 4: Run test to verify it passes**

Run: `cargo test -p logos-store-aletheia --test store_contract`  
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/logos-store-aletheia/src crates/logos-store-aletheia/tests
git commit -m "feat(store): add aletheia adapter contract for writes and reads"
```

### Task 10: Reporting Projections (CLI + TUI Inputs)

**Files:**
- Create: `crates/logos-reporting/src/net_worth.rs`
- Create: `crates/logos-reporting/src/cashflow.rs`
- Create: `crates/logos-reporting/src/budget_vs_actual.rs`
- Create: `crates/logos-reporting/src/register.rs`
- Create: `crates/logos-reporting/src/rsu_forecast.rs`
- Modify: `crates/logos-reporting/src/lib.rs`
- Test: `crates/logos-reporting/tests/reporting_smoke.rs`

**Step 1: Write failing report tests**

Build fixture data and assert core outputs are non-empty and numerically correct.

**Step 2: Run test to verify it fails**

Run: `cargo test -p logos-reporting --test reporting_smoke`  
Expected: FAIL.

**Step 3: Write minimal implementation**

Implement projection functions and output DTOs serializable to JSON.

**Step 4: Run test to verify it passes**

Run: `cargo test -p logos-reporting --test reporting_smoke`  
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/logos-reporting/src crates/logos-reporting/tests/reporting_smoke.rs
git commit -m "feat(reporting): add v0 core report projections"
```

### Task 11: Read-Only TUI Skeleton

**Files:**
- Create: `crates/logos-tui/src/app.rs`
- Create: `crates/logos-tui/src/ui/home.rs`
- Create: `crates/logos-tui/src/ui/budget.rs`
- Create: `crates/logos-tui/src/ui/register.rs`
- Create: `crates/logos-tui/src/ui/rsu.rs`
- Modify: `crates/logos-tui/src/main.rs`
- Test: `crates/logos-tui/tests/tui_smoke.rs`

**Step 1: Write failing smoke test**

Test app starts, builds one frame, exits cleanly.

**Step 2: Run test to verify it fails**

Run: `cargo test -p logos-tui --test tui_smoke`  
Expected: FAIL.

**Step 3: Write minimal implementation**

Implement:
- app state model
- read-only key bindings for navigation
- placeholder widgets fed by reporting layer interfaces

**Step 4: Run test to verify it passes**

Run: `cargo test -p logos-tui --test tui_smoke`  
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/logos-tui/src crates/logos-tui/tests/tui_smoke.rs
git commit -m "feat(tui): add read-only dashboard and navigation skeleton"
```

### Task 12: Verus Spine Proofs in `logos-proof`

**Files:**
- Modify: `logos-proof/transaction_invariants.verus`
- Create: `logos-proof/correction_invariants.verus`
- Create: `logos-proof/budget_invariants.verus`
- Create: `logos-proof/rsu_policy_invariants.verus`
- Create: `logos-proof/README.md`

**Step 1: Write failing proof goals**

Encode unproven assertions for:
- transaction balance invariant
- correction acyclic relation
- rollover conservation
- RSU policy validation properties

**Step 2: Run Verus to verify it fails**

Run: `C:\Users\markm\verus\verus.exe logos-proof/transaction_invariants.verus`  
Expected: FAIL with unproven obligations.

**Step 3: Write minimal proofs**

Add lemmas and proof blocks for each spine invariant.

**Step 4: Run Verus to verify it passes**

Run:
- `C:\Users\markm\verus\verus.exe logos-proof/transaction_invariants.verus`
- `C:\Users\markm\verus\verus.exe logos-proof/correction_invariants.verus`
- `C:\Users\markm\verus\verus.exe logos-proof/budget_invariants.verus`
- `C:\Users\markm\verus\verus.exe logos-proof/rsu_policy_invariants.verus`

Expected: all PASS.

**Step 5: Commit**

```bash
git add logos-proof
git commit -m "proof: add logos-proof spine invariants for v0 core"
```

### Task 13: End-to-End CLI Happy Path and Regression Coverage

**Files:**
- Create: `crates/logos-cli/tests/e2e_happy_path.rs`
- Create: `crates/logos-cli/tests/e2e_regressions.rs`
- Modify: `crates/logos-cli/src/commands/*.rs`

**Step 1: Write failing E2E tests**

Scenarios:
- create account(s), add balanced txn, run register report
- import CSV twice and assert no duplicate postings
- apply correction and verify latest-as-of read behavior

**Step 2: Run tests to verify failure**

Run: `cargo test -p logos-cli --test e2e_happy_path -- --nocapture`  
Expected: FAIL.

**Step 3: Implement minimal command wiring to pass**

Hook command handlers to core/store/import/reporting interfaces.

**Step 4: Re-run E2E tests**

Run:
- `cargo test -p logos-cli --test e2e_happy_path -- --nocapture`
- `cargo test -p logos-cli --test e2e_regressions -- --nocapture`

Expected: PASS.

**Step 5: Commit**

```bash
git add crates/logos-cli/src crates/logos-cli/tests
git commit -m "test(cli): add e2e happy path and idempotency regressions"
```

### Task 14: Quality Gates and Documentation Finish

**Files:**
- Modify: `docs/plans/2026-02-22-ledger-cli-tui-design.md` (implementation status notes)
- Create: `docs/adr/0001-v0-storage-and-proof-boundary.md`
- Create: `README.md`
- Modify: workspace `Cargo.toml` (if lint/tooling adjustments needed)

**Step 1: Add failing CI-equivalent checks locally**

Run:
- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all-targets --all-features`

Expected: at least one failure before cleanup.

**Step 2: Fix remaining issues**

Address formatting, lint, and test failures until all checks pass.

**Step 3: Re-run full gates**

Run:
- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all-targets --all-features`

Expected: all PASS.

**Step 4: Add docs**

- ADR explaining Aletheia-first v0 tradeoff and `logos-proof` boundary.
- README with setup, command examples, and current v0 limitations.

**Step 5: Commit**

```bash
git add README.md docs/adr docs/plans Cargo.toml
git commit -m "chore: pass quality gates and document v0 architecture decisions"
```

## Verification Checklist Before PR

- Run: `rg -n "TODO|FIXME|Stub:" crates logos-proof docs`
- Confirm no unresolved stubs in touched paths.
- Verify each write command has at least one happy-path integration test.
- Verify RSU planning-only path has no ledger mutation side effects.
- Verify correction flow is append-only in both CLI handler and store adapter.

## Suggested Milestone Tags

- Milestone A: Tasks 1-4 (ledger core write safety)
- Milestone B: Tasks 5-8 (budget + RSU + import)
- Milestone C: Tasks 9-11 (storage/read/tui)
- Milestone D: Tasks 12-14 (proofs + hardening)
