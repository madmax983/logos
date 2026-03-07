use std::collections::HashMap;

use logos_tui::{App, ReconcileDataSource, ReconcileRunRecord, ReconcileStatementLineRecord, View};

#[derive(Default)]
struct FakeReconcileSource {
    runs: Vec<ReconcileRunRecord>,
    lines_by_run: HashMap<String, Vec<ReconcileStatementLineRecord>>,
}

impl ReconcileDataSource for FakeReconcileSource {
    fn fetch_reconciliation_runs(
        &self,
        _month_key: Option<&str>,
        _checking_account: Option<&str>,
    ) -> Vec<ReconcileRunRecord> {
        self.runs.clone()
    }

    fn fetch_statement_lines_for_run(&self, run_id: &str) -> Vec<ReconcileStatementLineRecord> {
        self.lines_by_run.get(run_id).cloned().unwrap_or_default()
    }
}

#[test]
fn reconcile_view_renders_runs_and_statement_evidence() {
    let run = ReconcileRunRecord::new("recon-42", "2026-02", "assets:checking", 0, true, 2);
    let line = ReconcileStatementLineRecord::new("line-1", "2026-02-01", "COFFEE SHOP", -1_234);
    let source = FakeReconcileSource {
        runs: vec![run],
        lines_by_run: HashMap::from([(String::from("recon-42"), vec![line])]),
    };
    let mut app = App::new();
    app.set_view(View::Reconcile);
    app.set_reconcile_filters(Some("2026-02"), Some("assets:checking"));
    app.refresh_reconcile(&source);

    let frame = app.render_frame();

    assert!(frame.contains("Reconciliation View"));
    assert!(frame.contains("recon-42"));
    assert!(frame.contains("COFFEE SHOP"));
}

#[test]
fn reconcile_view_selection_navigation_switches_evidence_panel() {
    let run_a = ReconcileRunRecord::new("recon-1", "2026-02", "assets:checking", -500, false, 1);
    let run_b = ReconcileRunRecord::new("recon-2", "2026-02", "assets:checking", 0, true, 2);
    let source = FakeReconcileSource {
        runs: vec![run_a, run_b],
        lines_by_run: HashMap::from([
            (
                String::from("recon-1"),
                vec![ReconcileStatementLineRecord::new(
                    "line-a",
                    "2026-02-03",
                    "BOOK STORE",
                    -2_000,
                )],
            ),
            (
                String::from("recon-2"),
                vec![ReconcileStatementLineRecord::new(
                    "line-b",
                    "2026-02-04",
                    "PAYROLL",
                    100_000,
                )],
            ),
        ]),
    };
    let mut app = App::new();
    app.set_view(View::Reconcile);
    app.refresh_reconcile(&source);

    let first_frame = app.render_frame();
    assert!(first_frame.contains("BOOK STORE"));

    app.select_next_reconcile_run();
    let second_frame = app.render_frame();
    assert!(second_frame.contains("PAYROLL"));
}
