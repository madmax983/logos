use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use logos_cli::runtime::CliRuntime;
use logos_core::TransactionId;

fn temp_vault_path(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!("logos-capture-promote-{prefix}-{nanos}"))
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

fn runtime_from_note(raw_note: &str) -> CliRuntime {
    let fixture = temp_vault_path("capture");
    write_note(&fixture, "finance/inbox/2026/03/cap-1.md", raw_note);
    let mut runtime = CliRuntime::new_in_memory();
    runtime
        .ingest_capture_notes(&fixture, Some("finance/inbox"))
        .expect("ingest");
    cleanup_vault_path(&fixture);
    runtime
}

fn ready_capture_runtime() -> CliRuntime {
    runtime_from_note(
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
    )
}

fn unresolved_capture_runtime() -> CliRuntime {
    runtime_from_note(
        r#"---
capture_id: cap-1
captured_at: 2026-03-11T18:42:05Z
kind: expense
amount_cents: 1284
currency: USD
merchant_memo: Tacos El Rey
status: inbox
---
Team dinner
"#,
    )
}

#[test]
fn promote_posts_transaction_and_links_capture() {
    let mut runtime = ready_capture_runtime();
    let summary = runtime
        .promote_capture_draft("cap-1", None, None)
        .expect("promote");

    let txn_id = TransactionId::new(summary.transaction_id()).expect("txn id");
    assert!(runtime.transaction_exists(&txn_id));
    assert_eq!(summary.capture_id(), "cap-1");
    let draft = runtime.show_capture_draft("cap-1").expect("draft");
    assert_eq!(draft.status(), "promoted");
    assert_eq!(draft.promotion_txn_id(), Some(summary.transaction_id()));
}

#[test]
fn promote_rejects_already_promoted_draft() {
    let mut runtime = ready_capture_runtime();
    runtime
        .promote_capture_draft("cap-1", None, None)
        .expect("first promote");

    let err = runtime
        .promote_capture_draft("cap-1", None, None)
        .expect_err("second promote must fail");

    assert!(err.to_string().contains("already promoted"));
}

#[test]
fn promote_allows_account_overrides_when_rules_are_insufficient() {
    let mut runtime = unresolved_capture_runtime();
    let summary = runtime
        .promote_capture_draft(
            "cap-1",
            Some("expenses:food:dining"),
            Some("liabilities:amex:gold"),
        )
        .expect("promote with overrides");

    let draft = runtime.show_capture_draft("cap-1").expect("draft");
    assert_eq!(draft.status(), "promoted");
    assert_eq!(draft.promotion_txn_id(), Some(summary.transaction_id()));
}
