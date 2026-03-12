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
    assert_eq!(note.amount_cents, 1_284);
    assert_eq!(note.body.trim(), "Team dinner");
}

#[test]
fn rejects_markdown_without_frontmatter() {
    assert!(CaptureNote::parse("nope").is_err());
}

#[test]
fn rejects_unknown_capture_kind() {
    let raw = r#"---
capture_id: cap-1
captured_at: 2026-03-11T18:42:05Z
kind: goblin
amount_cents: 1284
currency: USD
merchant_memo: Tacos El Rey
status: inbox
---
Team dinner
"#;

    assert!(CaptureNote::parse(raw).is_err());
}
