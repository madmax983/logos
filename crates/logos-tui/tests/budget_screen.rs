use logos_tui::{App, BudgetDataSource, BudgetSnapshot, View};

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

#[test]
fn budget_view_renders_target_actual_and_variance() {
    let source = FakeBudgetSource {
        snapshot: Some(BudgetSnapshot::new(
            "2026-02",
            "expenses:",
            Some(300_000),
            1_019_776,
            Some(-719_776),
        )),
    };
    let mut app = App::new();
    app.set_view(View::Budget);
    app.refresh_budget(&source);

    let frame = app.render_frame();

    assert!(frame.contains("Budget View"));
    assert!(frame.contains("2026-02"));
    assert!(frame.contains("expenses:"));
    assert!(frame.contains("300000"));
    assert!(frame.contains("1019776"));
    assert!(frame.contains("-719776"));
}

#[test]
fn budget_view_renders_unconfigured_target_state() {
    let source = FakeBudgetSource {
        snapshot: Some(BudgetSnapshot::new(
            "2026-02",
            "expenses:",
            None,
            1_019_776,
            None,
        )),
    };
    let mut app = App::new();
    app.set_view(View::Budget);
    app.refresh_budget(&source);

    let frame = app.render_frame();

    assert!(frame.contains("Budget View"));
    assert!(frame.contains("unconfigured"));
    assert!(frame.contains("1019776"));
}

#[test]
fn budget_view_renders_unavailable_state_when_no_snapshot_loaded() {
    let source = FakeBudgetSource { snapshot: None };
    let mut app = App::new();
    app.set_view(View::Budget);
    app.refresh_budget(&source);

    let frame = app.render_frame();

    assert!(frame.contains("Budget View"));
    assert!(frame.contains("no budget data loaded"));
}
