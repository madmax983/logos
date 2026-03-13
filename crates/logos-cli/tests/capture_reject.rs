use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use logos_cli::runtime::CliRuntime;

fn temp_vault_path(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!("logos-capture-reject-{prefix}-{nanos}"))
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

fn seeded_capture_runtime() -> CliRuntime {
    let fixture = temp_vault_path("seeded");
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
Team dinner
"#,
    );

    let mut runtime = CliRuntime::new_in_memory();
    runtime
        .ingest_capture_notes(&fixture, Some("finance/inbox"))
        .expect("ingest");
    cleanup_vault_path(&fixture);
    runtime
}

#[test]
fn reject_marks_draft_terminal_with_reason() {
    let mut runtime = seeded_capture_runtime();
    runtime
        .reject_capture_draft("cap-1", "duplicate lunch")
        .expect("reject");
    let row = runtime.show_capture_draft("cap-1").expect("show");

    assert_eq!(row.status(), "rejected");
    assert_eq!(row.rejection_reason(), Some("duplicate lunch"));
}

#[test]
fn reject_rejects_already_promoted_draft() {
    let mut runtime = seeded_capture_runtime();
    runtime
        .promote_capture_draft(
            "cap-1",
            Some("expenses:food:dining"),
            Some("liabilities:amex:gold"),
        )
        .expect("promote");

    let err = runtime
        .reject_capture_draft("cap-1", "too late")
        .expect_err("promoted draft must stay terminal");

    assert!(err.to_string().contains("cannot be rejected"));
}
