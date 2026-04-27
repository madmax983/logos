use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use logos_tui::{
    App, AppInput, BudgetDataSource, BudgetSnapshot, HomeDataSource, HomeSnapshot,
    RegisterActivityRecord, RegisterDataSource, RegisterSnapshot, View, active_tab_index,
    key_event_to_app_input, runtime_unavailable_message, tab_titles, view_scope_lines,
    view_status_lines, view_title,
};

struct FakeHomeSource {
    snapshot: Option<HomeSnapshot>,
}

impl HomeDataSource for FakeHomeSource {
    fn fetch_home_snapshot(
        &self,
        _month_key: &str,
        _checking_account: &str,
        _expense_account_prefix: &str,
    ) -> Option<HomeSnapshot> {
        self.snapshot.clone()
    }
}

struct FakeBudgetSource {
    snapshot: Option<BudgetSnapshot>,
}

impl BudgetDataSource for FakeBudgetSource {
    fn fetch_budget_snapshot(
        &self,
        _month_key: &str,
        _expense_account_prefix: &str,
    ) -> Option<BudgetSnapshot> {
        self.snapshot.clone()
    }
}

struct FakeRegisterSource {
    snapshot: Option<RegisterSnapshot>,
}

impl RegisterDataSource for FakeRegisterSource {
    fn fetch_register_snapshot(&self, _account: &str) -> Option<RegisterSnapshot> {
        self.snapshot.clone()
    }
}

#[test]
fn terminal_key_mapping_supports_navigation_editing_and_quit_keys() {
    assert_eq!(
        key_event_to_app_input(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)),
        Some(AppInput::Cancel)
    );
    assert_eq!(
        key_event_to_app_input(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE)),
        Some(AppInput::Prev)
    );
    assert_eq!(
        key_event_to_app_input(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)),
        Some(AppInput::Next)
    );
    assert_eq!(
        key_event_to_app_input(KeyEvent::new(KeyCode::Left, KeyModifiers::NONE)),
        Some(AppInput::PrevView)
    );
    assert_eq!(
        key_event_to_app_input(KeyEvent::new(KeyCode::Right, KeyModifiers::NONE)),
        Some(AppInput::NextView)
    );
    assert_eq!(
        key_event_to_app_input(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE)),
        Some(AppInput::Next)
    );
    assert_eq!(
        key_event_to_app_input(KeyEvent::new(KeyCode::BackTab, KeyModifiers::SHIFT)),
        Some(AppInput::Prev)
    );
    assert_eq!(
        key_event_to_app_input(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE)),
        Some(AppInput::Backspace)
    );
    assert_eq!(
        key_event_to_app_input(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
        Some(AppInput::Submit)
    );
    assert_eq!(
        key_event_to_app_input(KeyEvent::new(KeyCode::Char('b'), KeyModifiers::NONE)),
        Some(AppInput::Char('b'))
    );
    assert_eq!(
        key_event_to_app_input(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL)),
        Some(AppInput::Quit)
    );
}

#[test]
fn runtime_unavailable_message_is_limited_to_data_views() {
    assert!(runtime_unavailable_message(View::Home).is_some());
    assert!(runtime_unavailable_message(View::Budget).is_some());
    assert!(runtime_unavailable_message(View::Register).is_some());
    assert!(runtime_unavailable_message(View::Reconcile).is_some());
    assert_eq!(runtime_unavailable_message(View::Rsu), None);
}

#[test]
fn view_title_tracks_active_screen() {
    assert_eq!(view_title(View::Home), "Home");
    assert_eq!(view_title(View::Budget), "Budget");
    assert_eq!(view_title(View::Register), "Register");
    assert_eq!(view_title(View::Rsu), "RSU");
    assert_eq!(view_title(View::Reconcile), "Reconcile");
}

#[test]
fn tabs_cover_all_views_and_select_active_index() {
    assert_eq!(
        tab_titles(),
        ["Home", "Budget", "Register", "RSU", "Reconcile"]
    );
    assert_eq!(active_tab_index(View::Home), 0);
    assert_eq!(active_tab_index(View::Budget), 1);
    assert_eq!(active_tab_index(View::Register), 2);
    assert_eq!(active_tab_index(View::Rsu), 3);
    assert_eq!(active_tab_index(View::Reconcile), 4);
}

#[test]
fn scope_and_status_lines_reflect_home_and_register_state() {
    let mut home_app = App::new();
    home_app.set_view(View::Home);
    home_app.refresh_home(&FakeHomeSource {
        snapshot: Some(HomeSnapshot::new(
            "2026-02",
            "assets:checking",
            "expenses:",
            77_048_806,
            78_068_582,
            1_019_776,
            77_048_806,
            Some(300_000),
            Some(-719_776),
        )),
    });

    let home_scope = view_scope_lines(&home_app);
    let home_status = view_status_lines(&home_app, true);
    assert!(
        home_scope
            .iter()
            .any(|line| line.contains("Month: 2026-02"))
    );
    assert!(
        home_scope
            .iter()
            .any(|line| line.contains("Checking: assets:checking"))
    );
    assert!(
        home_status
            .iter()
            .any(|line| line.contains("Cashflow: $770,488.06"))
    );
    assert!(
        home_status
            .iter()
            .any(|line| line.contains("Budget Target: $3,000.00"))
    );
    assert!(home_status.iter().any(|line| line.contains("Mode: Normal")));

    let mut register_app = App::new();
    register_app.set_view(View::Register);
    register_app.refresh_register(&FakeRegisterSource {
        snapshot: Some(RegisterSnapshot::new(
            "assets:checking",
            77_048_806,
            vec![
                RegisterActivityRecord::new("2026-02-28T14:42:12", "PAYROLL", 100_000),
                RegisterActivityRecord::new("2026-02-27T08:15:00", "COFFEE SHOP", -1_234),
            ],
        )),
    });

    let register_scope = view_scope_lines(&register_app);
    let register_status = view_status_lines(&register_app, true);
    assert_eq!(
        register_scope,
        vec![String::from("Account: assets:checking")]
    );
    assert!(
        register_status
            .iter()
            .any(|line| line.contains("Balance: $770,488.06"))
    );
    assert!(
        register_status
            .iter()
            .any(|line| line.contains("Recent Rows: 2"))
    );
    assert!(
        register_status
            .iter()
            .any(|line| line.contains("Mode: Normal"))
    );
}

#[test]
fn status_lines_surface_runtime_unavailability_for_data_views() {
    let mut app = App::new();
    app.set_view(View::Budget);
    app.refresh_budget(&FakeBudgetSource { snapshot: None });

    let status = view_status_lines(&app, false);

    assert!(
        status
            .iter()
            .any(|line| line.contains("Budget data unavailable"))
    );
    assert!(
        status
            .iter()
            .any(|line| line.contains("unable to initialize logos runtime"))
    );
    assert!(status.iter().any(|line| line.contains("Mode: Normal")));
}
