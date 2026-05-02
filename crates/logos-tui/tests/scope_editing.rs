use logos_tui::{App, AppInput, View, view_scope_lines, view_status_lines};

fn backspace_all(app: &mut App, value: &str) {
    for _ in 0..value.chars().count() {
        app.handle_input(AppInput::Backspace);
    }
}

#[test]
fn home_scope_editing_applies_drafts_on_submit() {
    let mut app = App::new();
    app.set_view(View::Home);

    app.handle_input(AppInput::Char('i'));
    assert!(app.is_scope_editing());

    let original_month = app.home_month_key().to_owned();
    backspace_all(&mut app, &original_month);
    for ch in "2026-02".chars() {
        app.handle_input(AppInput::Char(ch));
    }

    app.handle_input(AppInput::Next);
    let original_account = app.home_checking_account().to_owned();
    backspace_all(&mut app, &original_account);
    for ch in "assets:savings".chars() {
        app.handle_input(AppInput::Char(ch));
    }

    let scope_lines = view_scope_lines(&app);
    assert!(
        scope_lines
            .iter()
            .any(|line| line.contains("> Checking: assets:savings"))
    );
    assert!(view_status_lines(&app, true).iter().any(|line| {
        line.spans
            .iter()
            .any(|s| s.content.contains("Mode: Edit Scope"))
    }));

    app.handle_input(AppInput::Submit);

    assert!(!app.is_scope_editing());
    assert_eq!(app.home_month_key(), "2026-02");
    assert_eq!(app.home_checking_account(), "assets:savings");
}

#[test]
fn cancel_scope_edit_discards_register_draft() {
    let mut app = App::new();
    app.set_view(View::Register);
    let original = app.register_account().to_owned();

    app.handle_input(AppInput::Char('i'));
    assert!(app.is_scope_editing());

    backspace_all(&mut app, &original);
    for ch in "assets:wallet".chars() {
        app.handle_input(AppInput::Char(ch));
    }

    assert_eq!(
        view_scope_lines(&app),
        vec![String::from("> Account: assets:wallet")]
    );

    app.handle_input(AppInput::Cancel);

    assert!(!app.is_scope_editing());
    assert_eq!(app.register_account(), original);
    assert_eq!(
        view_scope_lines(&app),
        vec![String::from("Account: assets:checking")]
    );
}

#[test]
fn reconcile_scope_submit_normalizes_empty_drafts_to_wildcards() {
    let mut app = App::new();
    app.set_view(View::Reconcile);
    app.set_reconcile_filters(Some("2026-02"), Some("assets:checking"));

    app.handle_input(AppInput::Char('i'));
    assert!(app.is_scope_editing());

    backspace_all(&mut app, "2026-02");
    app.handle_input(AppInput::Next);
    backspace_all(&mut app, "assets:checking");

    app.handle_input(AppInput::Submit);

    assert!(!app.is_scope_editing());
    assert_eq!(app.reconcile_filter_month_key(), None);
    assert_eq!(app.reconcile_filter_checking_account(), None);
    assert_eq!(
        view_scope_lines(&app),
        vec![
            String::from("Month Filter: *"),
            String::from("Account Filter: *"),
        ]
    );
}

#[test]
fn left_and_right_inputs_cycle_views_in_normal_mode() {
    let mut app = App::new();
    assert_eq!(app.view(), View::Home);

    app.handle_input(AppInput::NextView);
    assert_eq!(app.view(), View::Budget);

    app.handle_input(AppInput::NextView);
    assert_eq!(app.view(), View::Register);

    app.handle_input(AppInput::PrevView);
    assert_eq!(app.view(), View::Budget);

    app.handle_input(AppInput::PrevView);
    assert_eq!(app.view(), View::Home);

    app.handle_input(AppInput::PrevView);
    assert_eq!(app.view(), View::Reconcile);
}

#[test]
fn invalid_home_month_keeps_editor_open_and_surfaces_error() {
    let mut app = App::new();
    app.set_view(View::Home);

    app.handle_input(AppInput::Char('i'));
    assert!(app.is_scope_editing());

    let original_month = app.home_month_key().to_owned();
    backspace_all(&mut app, &original_month);
    for ch in "2026-13".chars() {
        app.handle_input(AppInput::Char(ch));
    }

    app.handle_input(AppInput::Submit);

    assert!(app.is_scope_editing());
    assert_ne!(app.home_month_key(), "2026-13");
    assert!(view_status_lines(&app, true).iter().any(|line| {
        line.spans
            .iter()
            .any(|s| s.content.contains("Month must use YYYY-MM"))
    }));
}

#[test]
fn invalid_register_account_keeps_editor_open_and_surfaces_error() {
    let mut app = App::new();
    app.set_view(View::Register);

    app.handle_input(AppInput::Char('i'));
    assert!(app.is_scope_editing());

    let original = app.register_account().to_owned();
    backspace_all(&mut app, &original);
    for ch in "assets checking".chars() {
        app.handle_input(AppInput::Char(ch));
    }

    app.handle_input(AppInput::Submit);

    assert!(app.is_scope_editing());
    assert_eq!(app.register_account(), original);
    assert!(view_status_lines(&app, true).iter().any(|line| {
        line.spans
            .iter()
            .any(|s| s.content.contains("Account cannot contain spaces"))
    }));
}
