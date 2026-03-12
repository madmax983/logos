use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use logos_cli::runtime::CliRuntime;

fn temp_vault_path(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!("logos-capture-cli-{prefix}-{nanos}"))
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
    write_note(
        &fixture,
        "finance/inbox/2026/03/cap-2.md",
        r#"---
capture_id: cap-2
captured_at: 2026-03-12T08:00:00Z
kind: expense
amount_cents: 499
currency: USD
merchant_memo: Coffee House
status: inbox
---
Morning coffee
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
fn capture_list_filters_by_status() {
    let runtime = seeded_capture_runtime();
    let rows = runtime.list_capture_drafts(Some("inbox")).expect("list");

    assert_eq!(rows.len(), 2);
    assert!(rows.iter().all(|row| row.status() == "inbox"));
}

#[test]
fn capture_show_returns_one_draft() {
    let runtime = seeded_capture_runtime();
    let row = runtime.show_capture_draft("cap-1").expect("show");

    assert_eq!(row.capture_id(), "cap-1");
    assert_eq!(row.merchant_memo(), "Tacos El Rey");
}

#[test]
fn capture_list_rejects_unknown_status_filter() {
    let runtime = seeded_capture_runtime();
    let err = runtime
        .list_capture_drafts(Some("bogus-status"))
        .expect_err("invalid status must fail");

    assert!(err.to_string().contains("bogus-status"));
}
