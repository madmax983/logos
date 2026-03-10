# Logos Fetch Autopilot Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a new `logos-fetch` crate and integrate it into `month autopilot` so Logos can headlessly fetch institution statements, extract statement metadata, and continue month-close workflows with partial-failure handling.

**Architecture:** Keep accounting logic in the existing runtime/store crates and add a separate `logos-fetch` boundary for statement-source config, secret resolution, adapter execution, artifact staging, metadata extraction, and fetch-run status reporting. `logos-cli` will orchestrate fetch runs before import/reconcile, while institution-specific browser automation stays behind an adapter protocol so a broken site does not infect the ledger core.

**Tech Stack:** Rust 2024 workspace crate (`logos-fetch`), `tokio`, `serde`, `toml`, `thiserror`, `chrono`, deterministic artifact hashing, 1Password CLI (`op`) references for secrets, headless browser adapter protocol with fixture-driven tests first.

---

### Task 1: Record the Architecture Decision

**Files:**
- Create: `docs/adr/0003-headless-statement-fetch-and-autopilot-integration.md`
- Modify: `README.md`

**Step 1: Write the failing doc test**

Manually verify that no ADR currently documents statement-fetch architecture.

Run: `rg -n "statement fetch|1password|autopilot integration" docs/adr README.md`
Expected: No matching ADR covering this decision.

**Step 2: Run check to verify the gap exists**

Run: `rg -n "0003|headless statement fetch" docs/adr`
Expected: FAIL to find ADR `0003`.

**Step 3: Write minimal implementation**

Create an ADR capturing:
- why Plaid is out
- why browser automation is the v1 strategy
- why secrets are stored as 1Password references
- why partial success is first-class
- why `logos-fetch` is a separate crate instead of more `logos-cli` flags

Update `README.md` with one short paragraph pointing readers to the new fetch architecture.

**Step 4: Run verification**

Run: `rg -n "headless statement fetch|logos-fetch|1Password" docs/adr/0003-headless-statement-fetch-and-autopilot-integration.md README.md`
Expected: PASS with the new ADR and README reference.

**Step 5: Commit**

```bash
git add docs/adr/0003-headless-statement-fetch-and-autopilot-integration.md README.md
git commit -m "docs: record headless statement fetch architecture"
```

### Task 2: Scaffold the `logos-fetch` Workspace Crate

**Files:**
- Modify: `Cargo.toml`
- Create: `crates/logos-fetch/Cargo.toml`
- Create: `crates/logos-fetch/src/lib.rs`
- Create: `crates/logos-fetch/src/error.rs`
- Create: `crates/logos-fetch/src/model.rs`
- Create: `crates/logos-fetch/src/adapter.rs`
- Test: `crates/logos-fetch/tests/workspace_smoke.rs`

**Step 1: Write the failing test**

Create `crates/logos-fetch/tests/workspace_smoke.rs`:

```rust
#[test]
fn logos_fetch_crate_is_available() {
    assert!(logos_fetch::crate_ready());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p logos-fetch --test workspace_smoke`
Expected: FAIL with package `logos-fetch` not found.

**Step 3: Write minimal implementation**

Add the new workspace member and a tiny public API:

```rust
pub fn crate_ready() -> bool {
    true
}
```

Add `FetchError`, `StatementSource`, and `StatementAdapter` modules even if they only hold placeholders at first.

**Step 4: Run test to verify it passes**

Run: `cargo test -p logos-fetch --test workspace_smoke`
Expected: PASS.

**Step 5: Commit**

```bash
git add Cargo.toml crates/logos-fetch
git commit -m "build: scaffold logos-fetch crate"
```

### Task 3: Model Statement Sources and Desired Output Formats

**Files:**
- Modify: `crates/logos-fetch/src/lib.rs`
- Modify: `crates/logos-fetch/src/model.rs`
- Test: `crates/logos-fetch/tests/statement_source.rs`

**Step 1: Write the failing test**

Create `crates/logos-fetch/tests/statement_source.rs`:

```rust
use logos_fetch::{OutputFormat, StatementSource};

#[test]
fn statement_source_prefers_csv_before_pdf_when_configured() {
    let source = StatementSource::new(
        "amex:blue",
        "american-express",
        "assets:amex",
        vec![OutputFormat::Csv, OutputFormat::Pdf],
    )
    .expect("valid source");

    assert_eq!(source.preferred_format(), Some(OutputFormat::Csv));
}

#[test]
fn statement_source_rejects_empty_ids() {
    assert!(StatementSource::new("", "american-express", "assets:amex", vec![OutputFormat::Pdf]).is_err());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p logos-fetch --test statement_source`
Expected: FAIL with missing types/functions.

**Step 3: Write minimal implementation**

Implement:
- `OutputFormat::{Csv, Pdf}`
- `StatementSource` with `source_id`, `institution_id`, `ledger_account`, `format_preference`
- validation rejecting empty identifiers and empty format preference
- accessor for `preferred_format()`

**Step 4: Run test to verify it passes**

Run: `cargo test -p logos-fetch --test statement_source`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/logos-fetch/src crates/logos-fetch/tests/statement_source.rs
git commit -m "feat(fetch): add statement source model"
```

### Task 4: Parse Source Config From TOML Without Storing Raw Secrets

**Files:**
- Modify: `crates/logos-fetch/Cargo.toml`
- Modify: `crates/logos-fetch/src/lib.rs`
- Modify: `crates/logos-fetch/src/model.rs`
- Create: `crates/logos-fetch/src/config.rs`
- Test: `crates/logos-fetch/tests/config_parse.rs`

**Step 1: Write the failing test**

Create `crates/logos-fetch/tests/config_parse.rs`:

```rust
use logos_fetch::{OutputFormat, StatementSourceConfig};

#[test]
fn parses_source_config_with_1password_references() {
    let toml = r#"
        [[sources]]
        source_id = "pcu:checking"
        institution_id = "provident-credit-union"
        ledger_account = "assets:checking"
        format_preference = ["csv", "pdf"]
        username_secret_ref = "op://logos/provident/username"
        password_secret_ref = "op://logos/provident/password"
        totp_secret_ref = "op://logos/provident/totp"
    "#;

    let config = StatementSourceConfig::from_toml(toml).expect("config parses");
    assert_eq!(config.sources()[0].preferred_format(), Some(OutputFormat::Csv));
    assert_eq!(config.sources()[0].username_secret_ref(), "op://logos/provident/username");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p logos-fetch --test config_parse`
Expected: FAIL with parser/config missing.

**Step 3: Write minimal implementation**

Add `serde` + `toml` and implement:
- `StatementSourceConfig`
- `from_toml(&str) -> Result<Self, FetchError>`
- validation that secret fields are references, not empty raw values

**Step 4: Run test to verify it passes**

Run: `cargo test -p logos-fetch --test config_parse`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/logos-fetch/Cargo.toml crates/logos-fetch/src crates/logos-fetch/tests/config_parse.rs
git commit -m "feat(fetch): parse statement source config from toml"
```

### Task 5: Add Artifact Metadata and Partial-Failure Run Status

**Files:**
- Modify: `crates/logos-fetch/src/lib.rs`
- Modify: `crates/logos-fetch/src/model.rs`
- Create: `crates/logos-fetch/src/run.rs`
- Test: `crates/logos-fetch/tests/fetch_run.rs`

**Step 1: Write the failing test**

Create `crates/logos-fetch/tests/fetch_run.rs`:

```rust
use logos_fetch::{FetchRunStatus, FetchedStatementArtifact, OutputFormat};

#[test]
fn fetched_artifact_tracks_balances_and_format() {
    let artifact = FetchedStatementArtifact::new(
        "pcu:checking",
        OutputFormat::Pdf,
        "artifacts/statements/pcu-2026-02.pdf",
        "2026-02",
        100_00,
        250_00,
    )
    .expect("artifact");

    assert_eq!(artifact.month_key(), "2026-02");
    assert_eq!(artifact.closing_balance_cents(), 250_00);
}

#[test]
fn needs_attention_is_not_treated_as_terminal_success() {
    assert!(!FetchRunStatus::NeedsAttention.is_success());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p logos-fetch --test fetch_run`
Expected: FAIL with missing artifact/status types.

**Step 3: Write minimal implementation**

Implement:
- `FetchedStatementArtifact`
- `FetchRunStatus::{Downloaded, Imported, NoNewStatement, NeedsAttention, Failed}`
- `is_success()` helper
- validation for month key/path/source id

**Step 4: Run test to verify it passes**

Run: `cargo test -p logos-fetch --test fetch_run`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/logos-fetch/src crates/logos-fetch/tests/fetch_run.rs
git commit -m "feat(fetch): add artifact metadata and run statuses"
```

### Task 6: Define the Adapter Protocol With a Fake Adapter First

**Files:**
- Modify: `crates/logos-fetch/Cargo.toml`
- Modify: `crates/logos-fetch/src/lib.rs`
- Modify: `crates/logos-fetch/src/adapter.rs`
- Create: `crates/logos-fetch/src/secrets.rs`
- Test: `crates/logos-fetch/tests/fake_adapter.rs`

**Step 1: Write the failing test**

Create `crates/logos-fetch/tests/fake_adapter.rs`:

```rust
use logos_fetch::{
    FakeStatementAdapter, FetchRequest, FetchRunStatus, OutputFormat, SecretBundle, StatementSource,
};

#[tokio::test]
async fn fake_adapter_returns_downloaded_artifact_for_happy_path() {
    let source = StatementSource::new(
        "pcu:checking",
        "provident-credit-union",
        "assets:checking",
        vec![OutputFormat::Pdf],
    )
    .expect("source");
    let request = FetchRequest::new(&source, "2026-02");
    let secrets = SecretBundle::new("user", "pass", Some("123456")).expect("secrets");

    let result = FakeStatementAdapter::download_fixture_statement()
        .fetch(&request, &secrets)
        .await
        .expect("fetch");

    assert_eq!(result.status(), FetchRunStatus::Downloaded);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p logos-fetch --test fake_adapter`
Expected: FAIL with missing async adapter pieces.

**Step 3: Write minimal implementation**

Add:
- `StatementAdapter` trait
- `FetchRequest`
- `SecretBundle`
- `FakeStatementAdapter` returning deterministic fixture metadata
- `tokio` dependency for async tests

**Step 4: Run test to verify it passes**

Run: `cargo test -p logos-fetch --test fake_adapter`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/logos-fetch/Cargo.toml crates/logos-fetch/src crates/logos-fetch/tests/fake_adapter.rs
git commit -m "feat(fetch): define adapter protocol with fake adapter"
```

### Task 7: Add Runtime-Orchestrated Fetch Before Month Autopilot Import/Reconcile

**Files:**
- Modify: `crates/logos-cli/Cargo.toml`
- Modify: `crates/logos-cli/src/runtime.rs`
- Modify: `crates/logos-cli/src/commands/month.rs`
- Modify: `crates/logos-cli/src/commands/help.rs`
- Test: `crates/logos-cli/tests/e2e_happy_path.rs`

**Step 1: Write the failing test**

Add a new runtime-focused test in `crates/logos-cli/tests/e2e_happy_path.rs`:

```rust
#[test]
fn e2e_runtime_month_autopilot_uses_fetched_statement_metadata_when_no_balances_are_passed() {
    // Build runtime with fake fetch source config, run autopilot,
    // assert fetched opening/closing balances drive reconciliation.
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p logos-cli e2e_runtime_month_autopilot_uses_fetched_statement_metadata_when_no_balances_are_passed -- --exact`
Expected: FAIL because autopilot still requires typed balances and has no fetch integration.

**Step 3: Write minimal implementation**

Implement:
- runtime hook that resolves configured sources for the month
- adapter execution before import/reconcile
- automatic use of fetched balances when present
- help text updates for new config-driven fetch behavior

**Step 4: Run test to verify it passes**

Run: `cargo test -p logos-cli e2e_runtime_month_autopilot_uses_fetched_statement_metadata_when_no_balances_are_passed -- --exact`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/logos-cli/Cargo.toml crates/logos-cli/src crates/logos-cli/tests/e2e_happy_path.rs
git commit -m "feat(cli): run fetch sources before month autopilot reconciliation"
```

### Task 8: Persist and Surface Fetch-Run Outcomes for Partial Success

**Files:**
- Modify: `crates/logos-store-aletheia/src/model.rs`
- Modify: `crates/logos-store-aletheia/src/write.rs`
- Modify: `crates/logos-store-aletheia/src/read.rs`
- Modify: `crates/logos-cli/src/runtime.rs`
- Modify: `crates/logos-cli/src/args.rs`
- Create: `crates/logos-cli/src/commands/fetch.rs`
- Modify: `crates/logos-cli/src/commands/help.rs`
- Test: `crates/logos-store-aletheia/tests/store_contract.rs`
- Test: `crates/logos-cli/tests/cli_parse.rs`

**Step 1: Write the failing tests**

Add one store contract test and one CLI parse test:

```rust
#[test]
fn write_fetch_run_persists_needs_attention_status_and_error_summary() {
    // persist then reload fetch run
}

#[test]
fn parses_fetch_show_with_run_id() {
    // command path should be fetch.show
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test -p logos-store-aletheia write_fetch_run_persists_needs_attention_status_and_error_summary -- --exact`
Expected: FAIL because fetch runs are not in the store model.

Run: `cargo test -p logos-cli parses_fetch_show_with_run_id -- --exact`
Expected: FAIL because no fetch CLI exists.

**Step 3: Write minimal implementation**

Implement:
- stored fetch-run records and artifact links
- read APIs for list/show
- CLI `fetch list-runs` and `fetch show-run`
- autopilot reporting that distinguishes success from `needs_attention`

**Step 4: Run tests to verify they pass**

Run: `cargo test -p logos-store-aletheia write_fetch_run_persists_needs_attention_status_and_error_summary -- --exact`
Expected: PASS.

Run: `cargo test -p logos-cli parses_fetch_show_with_run_id -- --exact`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/logos-store-aletheia/src crates/logos-store-aletheia/tests/store_contract.rs crates/logos-cli/src crates/logos-cli/tests/cli_parse.rs
git commit -m "feat: persist and inspect statement fetch runs"
```

### Task 9: Add the First Real Institution Adapter Behind the Protocol

**Files:**
- Create: `crates/logos-fetch/src/adapters/mod.rs`
- Create: `crates/logos-fetch/src/adapters/provident.rs`
- Create: `crates/logos-fetch/tests/fixtures/provident_statement_metadata.json`
- Test: `crates/logos-fetch/tests/provident_adapter.rs`
- Optional later: `tools/logos-fetch-adapters/` for browser runner scripts

**Step 1: Write the failing test**

Create `crates/logos-fetch/tests/provident_adapter.rs`:

```rust
use logos_fetch::{FetchRequest, ProvidentAdapter, SecretBundle, StatementSource};

#[tokio::test]
async fn provident_adapter_maps_downloaded_statement_into_metadata() {
    // run against fixture runner output, assert source id/month/opening/closing balances
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p logos-fetch --test provident_adapter`
Expected: FAIL with missing adapter.

**Step 3: Write minimal implementation**

Implement the first real adapter against a fixture runner output contract before live-browser execution:
- parse adapter output JSON
- normalize statement month/path/balances
- map adapter failure states into `NeedsAttention` or `Failed`

**Step 4: Run test to verify it passes**

Run: `cargo test -p logos-fetch --test provident_adapter`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/logos-fetch/src/adapters crates/logos-fetch/tests/provident_adapter.rs crates/logos-fetch/tests/fixtures/provident_statement_metadata.json
git commit -m "feat(fetch): add provident statement adapter"
```

### Task 10: Add End-to-End Verification and Operator Docs

**Files:**
- Modify: `README.md`
- Create: `docs/fetch-ops.md`
- Modify: `crates/logos-cli/src/commands/help.rs`
- Test: `crates/logos-cli/tests/e2e_happy_path.rs`

**Step 1: Write the failing verification target**

Add an e2e test covering:
- two configured sources
- one fetched successfully
- one marked `needs_attention`
- autopilot continuing without closing the blocked account scope

**Step 2: Run test to verify it fails**

Run: `cargo test -p logos-cli e2e_runtime_month_autopilot_marks_partial_fetch_failure_as_needs_attention -- --exact`
Expected: FAIL until partial-success reporting exists.

**Step 3: Write minimal implementation**

Document:
- config file location and schema
- 1Password setup
- TOTP migration guidance from Google Authenticator to 1Password where possible
- Windows Task Scheduler/headless runner notes
- operator recovery path for `needs_attention`

**Step 4: Run test to verify it passes**

Run: `cargo test -p logos-cli e2e_runtime_month_autopilot_marks_partial_fetch_failure_as_needs_attention -- --exact`
Expected: PASS.

Run: `cargo test --workspace --offline`
Expected: PASS.

**Step 5: Commit**

```bash
git add README.md docs/fetch-ops.md crates/logos-cli/src/commands/help.rs crates/logos-cli/tests/e2e_happy_path.rs
git commit -m "docs: add fetch operations guide and end-to-end verification"
```
