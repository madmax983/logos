# Vault Capture Inbox Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a phone-friendly draft-capture workflow that ingests Obsidian-synced Markdown notes into Logos, classifies them, and promotes them into balanced ledger transactions through the existing posting path.

**Architecture:** Keep v1 local-first. Use the synced vault as a file transport only, persist capture drafts in `logos-store-aletheia`, parse and classify notes inside `logos-cli`, and reuse the existing runtime transaction writer for final posting. Source notes are never the authoritative workflow state after ingest; the store owns status and promotion linkage.

**Tech Stack:** Rust 2024 workspace crates, existing `logos-cli` and `logos-store-aletheia`, `serde`, `serde_yaml`, `chrono`, embedded Aletheia store, deterministic content hashing, Markdown file scanning via `std::fs`.

---

### Task 0: Restore a Clean Baseline Before Capture Work

**Files:**
- Modify: `crates/logos-cli/src/args.rs`

**Step 1: Reproduce the current baseline failure**

Run: `cargo test --workspace --offline`
Expected: FAIL with compile errors in `crates/logos-cli/src/args.rs` around `parse_required_parsed_flag` and missing `parse_optional_i64_value`.

**Step 2: Write the minimal compile fix**

Repair the pre-existing parser breakage in `crates/logos-cli/src/args.rs`:

- fix `parse_required_parsed_flag` so its return type matches the constructed `Ok(...)`
- restore or replace the missing optional `i64` parsing helper used by the caller
- keep behavior unchanged beyond restoring compilation

**Step 3: Verify the baseline is clean**

Run: `cargo test --workspace --offline`
Expected: PASS. If `trunk` has already advanced and this failure is gone, skip this task and note it in the execution log.

**Step 4: Commit**

```bash
git add crates/logos-cli/src/args.rs
git commit -m "fix(cli): restore argument parser baseline"
```

### Task 1: Add the Capture CLI Surface

**Files:**
- Modify: `crates/logos-cli/src/args.rs`
- Modify: `crates/logos-cli/src/commands/mod.rs`
- Modify: `crates/logos-cli/src/commands/help.rs`
- Create: `crates/logos-cli/src/commands/capture.rs`
- Test: `crates/logos-cli/tests/cli_parse.rs`

**Step 1: Write the failing tests**

Add CLI parse coverage in `crates/logos-cli/tests/cli_parse.rs`:

```rust
#[test]
fn parses_capture_ingest_command() {
    let args = vec![
        "ledger", "capture", "ingest", "--vault-path", "G:\\My Drive\\claude",
    ];
    let parsed = parse_args(args.into_iter().map(str::to_owned).collect()).expect("parse");
    assert_eq!(parsed.command_path(), "capture.ingest");
}

#[test]
fn parses_capture_promote_command() {
    let args = vec!["ledger", "capture", "promote", "--capture-id", "cap-1"];
    let parsed = parse_args(args.into_iter().map(str::to_owned).collect()).expect("parse");
    assert_eq!(parsed.command_path(), "capture.promote");
}
```

**Step 2: Run the targeted test**

Run: `cargo test -p logos-cli --test cli_parse -- --nocapture`
Expected: FAIL with unknown command/subcommand handling for `capture`.

**Step 3: Write the minimal implementation**

Add:

- `CaptureCommand::{Ingest, List, Show, Promote, Reject}` in `args.rs`
- parser branches for `ledger capture ...`
- `help.capture` text describing the new subcommands
- `commands::capture` module with handler functions that compile even if they temporarily delegate to unimplemented runtime methods

**Step 4: Re-run the targeted test**

Run: `cargo test -p logos-cli --test cli_parse -- --nocapture`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/logos-cli/src/args.rs crates/logos-cli/src/commands/mod.rs crates/logos-cli/src/commands/help.rs crates/logos-cli/src/commands/capture.rs crates/logos-cli/tests/cli_parse.rs
git commit -m "feat(cli): add capture command surface"
```

### Task 2: Persist Capture Drafts in the Store

**Files:**
- Modify: `crates/logos-store-aletheia/src/model.rs`
- Modify: `crates/logos-store-aletheia/src/lib.rs`
- Modify: `crates/logos-store-aletheia/src/write.rs`
- Modify: `crates/logos-store-aletheia/src/read.rs`
- Test: `crates/logos-store-aletheia/tests/store_contract.rs`

**Step 1: Write the failing tests**

Add tests to `crates/logos-store-aletheia/tests/store_contract.rs`:

```rust
#[test]
fn write_capture_draft_persists_and_can_be_listed() {
    let path = temp_store_path("capture-draft");
    let mut store = AletheiaStore::open(&path).expect("store");
    let draft = store
        .write_capture_draft(
            "cap-1",
            "G:/My Drive/claude/finance/inbox/2026/03/cap-1.md",
            "sha256:abc",
            "2026-03-11T18:42:05Z",
            "expense",
            1284,
            "USD",
            "Tacos El Rey",
            Some("liabilities:amex:gold"),
            None,
            Some("expenses:food:dining"),
            "Team dinner",
        )
        .expect("draft");

    assert_eq!(draft.capture_id(), "cap-1");
    assert_eq!(store.capture_draft_count(), 1);
}
```

**Step 2: Run the targeted test**

Run: `cargo test -p logos-store-aletheia --test store_contract write_capture_draft_persists_and_can_be_listed -- --exact`
Expected: FAIL with missing capture draft types and APIs.

**Step 3: Write the minimal implementation**

Add a new stored workflow artifact:

- `StoredCaptureDraft`
- `StoredCaptureStatus`
- write/read accessors for capture drafts
- persisted properties for capture id, source path, content hash, memo, hints, status, and optional `promotion_txn_id`

Mirror the existing fetch/reconciliation persistence pattern instead of inventing a second storage style.

**Step 4: Re-run the targeted test**

Run: `cargo test -p logos-store-aletheia --test store_contract write_capture_draft_persists_and_can_be_listed -- --exact`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/logos-store-aletheia/src/model.rs crates/logos-store-aletheia/src/lib.rs crates/logos-store-aletheia/src/write.rs crates/logos-store-aletheia/src/read.rs crates/logos-store-aletheia/tests/store_contract.rs
git commit -m "feat(store): persist capture drafts"
```

### Task 3: Parse Vault Notes With YAML Frontmatter

**Files:**
- Modify: `crates/logos-cli/Cargo.toml`
- Create: `crates/logos-cli/src/capture_note.rs`
- Modify: `crates/logos-cli/src/lib.rs`
- Test: `crates/logos-cli/tests/capture_note_parse.rs`

**Step 1: Write the failing tests**

Create `crates/logos-cli/tests/capture_note_parse.rs`:

```rust
use logos_cli::capture_note::CaptureNote;

#[test]
fn parses_markdown_capture_note_frontmatter() {
    let raw = r#"---
capture_id: cap-1
captured_at: 2026-03-11T18:42:05Z
kind: expense
amount_cents: 1284
currency: USD
merchant_memo: Tacos El Rey
status: inbox
---
Team dinner
"#;

    let note = CaptureNote::parse(raw).expect("note");
    assert_eq!(note.capture_id, "cap-1");
    assert_eq!(note.amount_cents, 1284);
    assert_eq!(note.body.trim(), "Team dinner");
}

#[test]
fn rejects_markdown_without_frontmatter() {
    assert!(CaptureNote::parse("nope").is_err());
}
```

**Step 2: Run the targeted test**

Run: `cargo test -p logos-cli --test capture_note_parse`
Expected: FAIL with missing parser/module.

**Step 3: Write the minimal implementation**

Add `serde_yaml` and implement:

- a `CaptureNoteFrontmatter` struct
- `CaptureNote::parse(&str) -> Result<Self, CaptureParseError>`
- frontmatter splitting on the first two `---` delimiters
- validation for required fields and accepted `kind`

Keep parsing strict and boring.

**Step 4: Re-run the targeted test**

Run: `cargo test -p logos-cli --test capture_note_parse`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/logos-cli/Cargo.toml crates/logos-cli/src/lib.rs crates/logos-cli/src/capture_note.rs crates/logos-cli/tests/capture_note_parse.rs
git commit -m "feat(capture): parse vault markdown notes"
```

### Task 4: Implement Ingest File Discovery and Idempotent Runtime Writes

**Files:**
- Modify: `crates/logos-cli/src/runtime.rs`
- Modify: `crates/logos-cli/src/commands/capture.rs`
- Modify: `crates/logos-cli/src/capture_note.rs`
- Test: `crates/logos-cli/tests/capture_ingest.rs`

**Step 1: Write the failing tests**

Create `crates/logos-cli/tests/capture_ingest.rs`:

```rust
#[test]
fn ingest_reads_markdown_files_and_persists_one_draft() {
    let fixture = tempdir().expect("tempdir");
    std::fs::create_dir_all(fixture.path().join("finance/inbox/2026/03")).expect("mkdir");
    std::fs::write(
        fixture.path().join("finance/inbox/2026/03/cap-1.md"),
        r#"---
capture_id: cap-1
captured_at: 2026-03-11T18:42:05Z
kind: expense
amount_cents: 1284
currency: USD
merchant_memo: Tacos El Rey
status: inbox
---
"#,
    )
    .expect("write");

    let mut runtime = test_runtime();
    let summary = runtime
        .ingest_capture_notes(fixture.path(), Some("finance/inbox"))
        .expect("ingest");

    assert_eq!(summary.ingested_count(), 1);
    assert_eq!(summary.conflict_count(), 0);
}
```

**Step 2: Run the targeted test**

Run: `cargo test -p logos-cli --test capture_ingest ingest_reads_markdown_files_and_persists_one_draft -- --exact`
Expected: FAIL with missing runtime ingest API.

**Step 3: Write the minimal implementation**

Add runtime support for:

- recursive inbox scan of `*.md`
- note parsing through `capture_note`
- deterministic content hashing
- idempotent write/update semantics keyed by `capture_id`
- ingest summary counts for `ingested`, `updated`, `skipped`, `conflict`, and `malformed`

Wire `ledger capture ingest` to print a deterministic summary line, similar to existing CLI commands.

**Step 4: Re-run the targeted test**

Run: `cargo test -p logos-cli --test capture_ingest ingest_reads_markdown_files_and_persists_one_draft -- --exact`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/logos-cli/src/runtime.rs crates/logos-cli/src/commands/capture.rs crates/logos-cli/src/capture_note.rs crates/logos-cli/tests/capture_ingest.rs
git commit -m "feat(capture): ingest vault inbox drafts"
```

### Task 5: Add `capture list` and `capture show`

**Files:**
- Modify: `crates/logos-cli/src/runtime.rs`
- Modify: `crates/logos-cli/src/commands/capture.rs`
- Modify: `crates/logos-cli/src/commands/help.rs`
- Test: `crates/logos-cli/tests/capture_cli.rs`

**Step 1: Write the failing tests**

Create `crates/logos-cli/tests/capture_cli.rs`:

```rust
#[test]
fn capture_list_filters_by_status() {
    let mut runtime = seeded_capture_runtime();
    let rows = runtime.list_capture_drafts(Some("needs_review")).expect("list");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].status(), "needs_review");
}

#[test]
fn capture_show_returns_one_draft() {
    let mut runtime = seeded_capture_runtime();
    let row = runtime.show_capture_draft("cap-1").expect("show");
    assert_eq!(row.capture_id(), "cap-1");
}
```

**Step 2: Run the targeted test**

Run: `cargo test -p logos-cli --test capture_cli`
Expected: FAIL with missing list/show runtime methods.

**Step 3: Write the minimal implementation**

Add:

- store-backed runtime read methods
- command rendering for `capture list` and `capture show`
- optional status filtering
- help text for the new read surface

Keep output deterministic and scriptable.

**Step 4: Re-run the targeted test**

Run: `cargo test -p logos-cli --test capture_cli`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/logos-cli/src/runtime.rs crates/logos-cli/src/commands/capture.rs crates/logos-cli/src/commands/help.rs crates/logos-cli/tests/capture_cli.rs
git commit -m "feat(capture): add list and show commands"
```

### Task 6: Classify Drafts With Minimal Suggestion Rules

**Files:**
- Create: `crates/logos-cli/src/capture_rules.rs`
- Modify: `crates/logos-cli/src/lib.rs`
- Modify: `crates/logos-cli/src/runtime.rs`
- Test: `crates/logos-cli/tests/capture_rules.rs`

**Step 1: Write the failing tests**

Create `crates/logos-cli/tests/capture_rules.rs`:

```rust
#[test]
fn exact_merchant_match_marks_draft_ready() {
    let mut runtime = runtime_with_prior_tacos_history();
    let result = runtime.reclassify_capture_draft("cap-1").expect("classify");
    assert_eq!(result.status(), "ready");
    assert_eq!(result.suggested_debit_account(), Some("expenses:food:dining"));
    assert_eq!(result.suggested_credit_account(), Some("liabilities:amex:gold"));
}

#[test]
fn missing_history_keeps_draft_in_needs_review() {
    let mut runtime = test_runtime();
    let result = runtime.reclassify_capture_draft("cap-unknown").expect("classify");
    assert_eq!(result.status(), "needs_review");
}
```

**Step 2: Run the targeted test**

Run: `cargo test -p logos-cli --test capture_rules`
Expected: FAIL with missing rule engine.

**Step 3: Write the minimal implementation**

Implement narrow v1 rules:

- normalize merchant memo for exact case-insensitive matching
- use explicit note hints first
- fall back to per-kind defaults where available
- infer accounts from prior matching transaction descriptions
- emit `ready`, `suggested`, or `needs_review`

Do not add fuzzy matching in this slice.

**Step 4: Re-run the targeted test**

Run: `cargo test -p logos-cli --test capture_rules`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/logos-cli/src/capture_rules.rs crates/logos-cli/src/lib.rs crates/logos-cli/src/runtime.rs crates/logos-cli/tests/capture_rules.rs
git commit -m "feat(capture): classify drafts from history and hints"
```

### Task 7: Promote Drafts Into Real Transactions

**Files:**
- Modify: `crates/logos-cli/src/runtime.rs`
- Modify: `crates/logos-cli/src/commands/capture.rs`
- Modify: `crates/logos-store-aletheia/src/write.rs`
- Modify: `crates/logos-store-aletheia/src/read.rs`
- Test: `crates/logos-cli/tests/capture_promote.rs`
- Test: `crates/logos-store-aletheia/tests/store_contract.rs`

**Step 1: Write the failing tests**

Create `crates/logos-cli/tests/capture_promote.rs`:

```rust
#[test]
fn promote_posts_transaction_and_links_capture() {
    let mut runtime = ready_capture_runtime();
    let summary = runtime.promote_capture_draft("cap-1", None, None).expect("promote");

    assert!(summary.transaction_id().starts_with("txn-"));
    assert_eq!(summary.capture_id(), "cap-1");
    assert_eq!(runtime.show_capture_draft("cap-1").expect("draft").status(), "promoted");
}

#[test]
fn promote_rejects_already_promoted_draft() {
    let mut runtime = already_promoted_capture_runtime();
    assert!(runtime.promote_capture_draft("cap-1", None, None).is_err());
}
```

**Step 2: Run the targeted test**

Run: `cargo test -p logos-cli --test capture_promote`
Expected: FAIL with missing promote path or missing store linkage.

**Step 3: Write the minimal implementation**

Implement:

- runtime promotion method that resolves accounts, then calls `post_double_entry`
- store update to mark the draft `promoted`
- optional override flags for debit/credit accounts
- guard against double-promotion

Reuse existing transaction posting instead of building a second mutation pipeline.

**Step 4: Re-run the targeted test**

Run: `cargo test -p logos-cli --test capture_promote`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/logos-cli/src/runtime.rs crates/logos-cli/src/commands/capture.rs crates/logos-store-aletheia/src/write.rs crates/logos-store-aletheia/src/read.rs crates/logos-cli/tests/capture_promote.rs crates/logos-store-aletheia/tests/store_contract.rs
git commit -m "feat(capture): promote drafts into ledger transactions"
```

### Task 8: Add Explicit Reject and Conflict Handling

**Files:**
- Modify: `crates/logos-cli/src/runtime.rs`
- Modify: `crates/logos-cli/src/commands/capture.rs`
- Modify: `crates/logos-store-aletheia/src/write.rs`
- Test: `crates/logos-cli/tests/capture_reject.rs`
- Test: `crates/logos-cli/tests/capture_ingest.rs`

**Step 1: Write the failing tests**

Add tests:

```rust
#[test]
fn reject_marks_draft_terminal_with_reason() {
    let mut runtime = seeded_capture_runtime();
    runtime.reject_capture_draft("cap-1", "duplicate lunch").expect("reject");
    let row = runtime.show_capture_draft("cap-1").expect("show");
    assert_eq!(row.status(), "rejected");
    assert_eq!(row.rejection_reason(), Some("duplicate lunch"));
}

#[test]
fn changed_payload_after_promotion_becomes_conflict() {
    // ingest, promote, then re-ingest same capture_id with different amount
}
```

**Step 2: Run the targeted tests**

Run: `cargo test -p logos-cli --test capture_reject`
Expected: FAIL with missing reject/conflict logic.

**Step 3: Write the minimal implementation**

Add:

- explicit reject command and runtime method
- store mutation for `rejected` state plus reason
- ingest-time conflict handling when a terminal draft reappears with changed content

Do not silently overwrite terminal states.

**Step 4: Re-run the targeted tests**

Run: `cargo test -p logos-cli --test capture_reject`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/logos-cli/src/runtime.rs crates/logos-cli/src/commands/capture.rs crates/logos-store-aletheia/src/write.rs crates/logos-cli/tests/capture_reject.rs crates/logos-cli/tests/capture_ingest.rs
git commit -m "feat(capture): add reject and conflict handling"
```

### Task 9: Document the Obsidian Workflow and Template

**Files:**
- Create: `docs/capture-ops.md`
- Create: `docs/templates/obsidian-capture-note.md`
- Modify: `README.md`

**Step 1: Write the failing doc check**

Run: `rg -n "capture ingest|Obsidian|vault inbox" README.md docs`
Expected: No operator docs for the new capture workflow.

**Step 2: Write the minimal documentation**

Document:

- recommended vault folder layout
- example note template
- ingest/list/promote workflow
- environment variables
- common failure modes like malformed frontmatter and sync delay

Add one README paragraph linking to the new docs.

**Step 3: Verify the docs**

Run: `rg -n "vault inbox|capture ingest|obsidian mobile" README.md docs/capture-ops.md docs/templates/obsidian-capture-note.md`
Expected: PASS.

**Step 4: Commit**

```bash
git add docs/capture-ops.md docs/templates/obsidian-capture-note.md README.md
git commit -m "docs: add vault capture workflow"
```

### Task 10: Run Final Verification and Review the Whole Slice

**Files:**
- Modify: any touched files from prior tasks if review finds issues

**Step 1: Run targeted tests**

Run:

```bash
cargo test -p logos-store-aletheia --test store_contract
cargo test -p logos-cli --test cli_parse
cargo test -p logos-cli --test capture_note_parse
cargo test -p logos-cli --test capture_ingest
cargo test -p logos-cli --test capture_cli
cargo test -p logos-cli --test capture_rules
cargo test -p logos-cli --test capture_promote
cargo test -p logos-cli --test capture_reject
```

Expected: PASS.

**Step 2: Run workspace verification**

Run:

```bash
cargo fmt --all
cargo test --workspace --offline
rg -n "TODO|FIXME|Stub:" crates/logos-cli crates/logos-store-aletheia docs/capture-ops.md docs/templates/obsidian-capture-note.md
```

Expected: `cargo fmt` succeeds, tests pass, and ripgrep finds no placeholder garbage.

**Step 3: Review the diff**

Run:

```bash
git status --short
git diff --stat
git diff -- crates/logos-cli crates/logos-store-aletheia README.md docs
```

Expected: coherent capture workflow changes only.

**Step 4: Commit**

```bash
git add crates/logos-cli crates/logos-store-aletheia README.md docs
git commit -m "feat(capture): add vault-backed transaction draft workflow"
```
