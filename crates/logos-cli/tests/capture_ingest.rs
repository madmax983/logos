use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use logos_cli::runtime::CliRuntime;

fn temp_vault_path(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!("logos-capture-vault-{prefix}-{nanos}"))
}

fn cleanup_vault_path(path: &Path) {
    if path.exists() {
        let _ = std::fs::remove_dir_all(path);
    }
}

fn write_note(vault_root: &Path, relative_path: &str, raw: &str) {
    let full_path = vault_root.join(relative_path);
    let parent = full_path.parent().expect("parent");
    std::fs::create_dir_all(parent).expect("mkdir");
    std::fs::write(full_path, raw).expect("write note");
}

#[test]
fn ingest_reads_markdown_files_and_persists_one_draft() {
    let fixture = temp_vault_path("single");
    write_note(
        &fixture,
        "finance/inbox/2026/03/cap-1.md",
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
    );

    let mut runtime = CliRuntime::new_in_memory();
    let summary = runtime
        .ingest_capture_notes(&fixture, Some("finance/inbox"))
        .expect("ingest");

    assert_eq!(summary.ingested_count(), 1);
    assert_eq!(summary.updated_count(), 0);
    assert_eq!(summary.skipped_count(), 0);
    assert_eq!(summary.conflict_count(), 0);
    assert_eq!(summary.malformed_count(), 0);

    cleanup_vault_path(&fixture);
}

#[test]
fn ingest_reingest_of_identical_note_is_skipped() {
    let fixture = temp_vault_path("skip");
    let raw = r#"---
capture_id: cap-1
captured_at: 2026-03-11T18:42:05Z
kind: expense
amount_cents: 1284
currency: USD
merchant_memo: Tacos El Rey
status: inbox
---
"#;
    write_note(&fixture, "finance/inbox/2026/03/cap-1.md", raw);

    let mut runtime = CliRuntime::new_in_memory();
    runtime
        .ingest_capture_notes(&fixture, Some("finance/inbox"))
        .expect("initial ingest");

    let summary = runtime
        .ingest_capture_notes(&fixture, Some("finance/inbox"))
        .expect("reingest");

    assert_eq!(summary.ingested_count(), 0);
    assert_eq!(summary.updated_count(), 0);
    assert_eq!(summary.skipped_count(), 1);
    assert_eq!(summary.conflict_count(), 0);
    assert_eq!(summary.malformed_count(), 0);

    cleanup_vault_path(&fixture);
}

#[test]
fn ingest_changed_note_updates_existing_non_terminal_draft() {
    let fixture = temp_vault_path("update");
    write_note(
        &fixture,
        "finance/inbox/2026/03/cap-1.md",
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
    );

    let mut runtime = CliRuntime::new_in_memory();
    runtime
        .ingest_capture_notes(&fixture, Some("finance/inbox"))
        .expect("initial ingest");

    write_note(
        &fixture,
        "finance/inbox/2026/03/cap-1.md",
        r#"---
capture_id: cap-1
captured_at: 2026-03-11T18:42:05Z
kind: expense
amount_cents: 1284
currency: USD
merchant_memo: Tacos El Primo
status: inbox
---
Updated memo
"#,
    );

    let summary = runtime
        .ingest_capture_notes(&fixture, Some("finance/inbox"))
        .expect("updated ingest");

    assert_eq!(summary.ingested_count(), 0);
    assert_eq!(summary.updated_count(), 1);
    assert_eq!(summary.skipped_count(), 0);
    assert_eq!(summary.conflict_count(), 0);
    assert_eq!(summary.malformed_count(), 0);

    cleanup_vault_path(&fixture);
}

#[test]
fn ingest_counts_malformed_notes_and_continues() {
    let fixture = temp_vault_path("malformed");
    write_note(
        &fixture,
        "finance/inbox/2026/03/cap-1.md",
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
    );
    write_note(&fixture, "finance/inbox/2026/03/bad.md", "nope");

    let mut runtime = CliRuntime::new_in_memory();
    let summary = runtime
        .ingest_capture_notes(&fixture, Some("finance/inbox"))
        .expect("ingest");

    assert_eq!(summary.ingested_count(), 1);
    assert_eq!(summary.updated_count(), 0);
    assert_eq!(summary.skipped_count(), 0);
    assert_eq!(summary.conflict_count(), 0);
    assert_eq!(summary.malformed_count(), 1);

    cleanup_vault_path(&fixture);
}

#[test]
fn changed_payload_after_promotion_becomes_conflict() {
    let fixture = temp_vault_path("terminal-conflict");
    write_note(
        &fixture,
        "finance/inbox/2026/03/cap-1.md",
        r#"---
capture_id: cap-1
captured_at: 2026-03-11T18:42:05Z
kind: expense
amount_cents: 1284
currency: USD
merchant_memo: Tacos El Rey
from_account_hint: liabilities:amex:gold
category_hint: expenses:food:dining
status: inbox
---
Team dinner
"#,
    );

    let mut runtime = CliRuntime::new_in_memory();
    runtime
        .ingest_capture_notes(&fixture, Some("finance/inbox"))
        .expect("initial ingest");
    let promoted = runtime
        .promote_capture_draft("cap-1", None, None)
        .expect("promote");

    write_note(
        &fixture,
        "finance/inbox/2026/03/cap-1.md",
        r#"---
capture_id: cap-1
captured_at: 2026-03-11T18:42:05Z
kind: expense
amount_cents: 1584
currency: USD
merchant_memo: Tacos El Rey
from_account_hint: liabilities:amex:gold
category_hint: expenses:food:dining
status: inbox
---
Team dinner with dessert
"#,
    );

    let summary = runtime
        .ingest_capture_notes(&fixture, Some("finance/inbox"))
        .expect("conflict ingest");
    let row = runtime.show_capture_draft("cap-1").expect("show");

    assert_eq!(summary.ingested_count(), 0);
    assert_eq!(summary.updated_count(), 0);
    assert_eq!(summary.skipped_count(), 0);
    assert_eq!(summary.conflict_count(), 1);
    assert_eq!(summary.malformed_count(), 0);
    assert_eq!(row.status(), "conflict");
    assert_eq!(row.amount_cents(), 1_584);
    assert_eq!(row.promotion_txn_id(), Some(promoted.transaction_id()));
    assert!(
        row.rejection_reason()
            .is_some_and(|value| value.contains("terminal"))
    );

    cleanup_vault_path(&fixture);
}
