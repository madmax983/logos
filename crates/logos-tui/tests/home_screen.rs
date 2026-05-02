use logos_tui::{App, HomeDataSource, HomeSnapshot, View};

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

#[test]
fn home_view_renders_month_cashflow_and_budget_summary() {
    let source = FakeHomeSource {
        snapshot: Some(HomeSnapshot::new(
            "2026-02",
            "assets:checking",
            "expenses:test:",
            11_111_111,
            22_222_222,
            33_333_333,
            44_444_444,
            Some(55_555_555),
            Some(66_666_666),
        )),
    };
    let mut app = App::new();
    app.set_view(View::Home);
    app.refresh_home(&source);

    let frame = app.render_frame();

    assert!(frame.contains("Home Dashboard"));
    assert!(frame.contains("2026-02"));
    assert!(frame.contains("assets:checking"));
    assert!(frame.contains("$111,111.11"));
    assert!(frame.contains("expenses:test:"));
    assert!(frame.contains("$222,222.22"));
    assert!(frame.contains("$333,333.33"));
    assert!(frame.contains("$444,444.44"));
    assert!(frame.contains("$555,555.55"));
    assert!(frame.contains("$666,666.66"));
}

#[test]
fn home_view_renders_unavailable_state_when_no_snapshot_loaded() {
    let source = FakeHomeSource { snapshot: None };
    let mut app = App::new();
    app.set_view(View::Home);
    app.refresh_home(&source);

    let frame = app.render_frame();

    assert!(frame.contains("Home Dashboard"));
    assert!(frame.contains("no dashboard data loaded"));
}
