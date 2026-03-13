use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use logos_cli::runtime::CliRuntime;

fn temp_vault_path(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!("logos-capture-rules-{prefix}-{nanos}"))
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

fn runtime_with_capture(raw_note: &str) -> CliRuntime {
    let fixture = temp_vault_path("capture");
    write_note(&fixture, "finance/inbox/2026/03/cap-1.md", raw_note);
    let mut runtime = CliRuntime::new_in_memory();
    runtime
        .ingest_capture_notes(&fixture, Some("finance/inbox"))
        .expect("ingest");
    cleanup_vault_path(&fixture);
    runtime
}

fn runtime_with_prior_tacos_history() -> CliRuntime {
    let mut runtime = runtime_with_capture(
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
    runtime
        .post_double_entry(
            "TACOS EL REY",
            "expenses:food:dining",
            "liabilities:amex:gold",
            1_500,
        )
        .expect("history");
    runtime
}

#[test]
fn exact_merchant_match_marks_draft_ready() {
    let runtime = runtime_with_prior_tacos_history();
    let result = runtime.reclassify_capture_draft("cap-1").expect("classify");

    assert_eq!(result.status(), "ready");
    assert_eq!(
        result.suggested_debit_account(),
        Some("expenses:food:dining")
    );
    assert_eq!(
        result.suggested_credit_account(),
        Some("liabilities:amex:gold")
    );
}

#[test]
fn missing_history_keeps_draft_in_needs_review() {
    let runtime = runtime_with_capture(
        r#"---
capture_id: cap-unknown
captured_at: 2026-03-11T18:42:05Z
kind: expense
amount_cents: 1284
currency: USD
merchant_memo: Unknown Vendor
status: inbox
---
"#,
    );
    let result = runtime
        .reclassify_capture_draft("cap-unknown")
        .expect("classify");

    assert_eq!(result.status(), "needs_review");
    assert_eq!(result.suggested_debit_account(), None);
    assert_eq!(result.suggested_credit_account(), None);
}

#[test]
fn explicit_hints_win_before_history() {
    let runtime = runtime_with_capture(
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
"#,
    );
    let result = runtime.reclassify_capture_draft("cap-1").expect("classify");

    assert_eq!(result.status(), "ready");
    assert_eq!(
        result.suggested_debit_account(),
        Some("expenses:food:dining")
    );
    assert_eq!(
        result.suggested_credit_account(),
        Some("liabilities:amex:gold")
    );
}
