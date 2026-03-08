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
            "expenses:",
            77_048_806,
            78_068_582,
            1_019_776,
            77_048_806,
            Some(300_000),
            Some(-719_776),
        )),
    };
    let mut app = App::new();
    app.set_view(View::Home);
    app.refresh_home(&source);

    let frame = app.render_frame();

    assert!(frame.contains("Home Dashboard"));
    assert!(frame.contains("2026-02"));
    assert!(frame.contains("assets:checking"));
    assert!(frame.contains("77048806"));
    assert!(frame.contains("78068582"));
    assert!(frame.contains("1019776"));
    assert!(frame.contains("300000"));
    assert!(frame.contains("-719776"));
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
