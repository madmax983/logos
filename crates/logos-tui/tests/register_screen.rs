use logos_tui::{App, RegisterActivityRecord, RegisterDataSource, RegisterSnapshot, View};

struct FakeRegisterSource {
    snapshot: Option<RegisterSnapshot>,
}

impl RegisterDataSource for FakeRegisterSource {
    fn fetch_register_snapshot(&self, _account: &str) -> Option<RegisterSnapshot> {
        self.snapshot.clone()
    }
}

#[test]
fn register_view_renders_balance_and_recent_activity() {
    let source = FakeRegisterSource {
        snapshot: Some(RegisterSnapshot::new(
            "assets:checking",
            77_048_806,
            vec![
                RegisterActivityRecord::new("2026-02-28T00:00:00", "PAYROLL", 100_000),
                RegisterActivityRecord::new("2026-02-27T00:00:00", "COFFEE SHOP", -1_234),
            ],
        )),
    };
    let mut app = App::new();
    app.set_view(View::Register);
    app.refresh_register(&source);

    let frame = app.render_frame();

    assert!(frame.contains("Register View"));
    assert!(frame.contains("assets:checking"));
    assert!(frame.contains("$770,488.06"));
    assert!(frame.contains("PAYROLL"));
    assert!(frame.contains("COFFEE SHOP"));
    assert!(frame.contains("-$12.34"));
}

#[test]
fn register_view_renders_empty_activity_state() {
    let source = FakeRegisterSource {
        snapshot: Some(RegisterSnapshot::new("assets:checking", 0, Vec::new())),
    };
    let mut app = App::new();
    app.set_view(View::Register);
    app.refresh_register(&source);

    let frame = app.render_frame();

    assert!(frame.contains("Register View"));
    assert!(frame.contains("assets:checking"));
    assert!(frame.contains("$0.00"));
    assert!(frame.contains("no activity for selected account"));
}
