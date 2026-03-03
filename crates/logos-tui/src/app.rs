use std::collections::HashMap;

use logos_app::CliRuntime;

use crate::ui::{budget, home, reconcile, register, rsu};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Home,
    Budget,
    Register,
    Rsu,
    Reconcile,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReconcileRunRecord {
    run_id: String,
    month_key: String,
    checking_account: String,
    variance_cents: i64,
    reconciled: bool,
    matched_transaction_count: i64,
}

impl ReconcileRunRecord {
    #[must_use]
    pub fn new(
        run_id: &str,
        month_key: &str,
        checking_account: &str,
        variance_cents: i64,
        reconciled: bool,
        matched_transaction_count: i64,
    ) -> Self {
        Self {
            run_id: run_id.to_owned(),
            month_key: month_key.to_owned(),
            checking_account: checking_account.to_owned(),
            variance_cents,
            reconciled,
            matched_transaction_count,
        }
    }

    #[must_use]
    pub fn run_id(&self) -> &str {
        &self.run_id
    }

    #[must_use]
    pub fn month_key(&self) -> &str {
        &self.month_key
    }

    #[must_use]
    pub fn checking_account(&self) -> &str {
        &self.checking_account
    }

    #[must_use]
    pub const fn variance_cents(&self) -> i64 {
        self.variance_cents
    }

    #[must_use]
    pub const fn reconciled(&self) -> bool {
        self.reconciled
    }

    #[must_use]
    pub const fn matched_transaction_count(&self) -> i64 {
        self.matched_transaction_count
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReconcileStatementLineRecord {
    line_id: String,
    statement_timestamp: String,
    memo: String,
    amount_cents: i64,
}

impl ReconcileStatementLineRecord {
    #[must_use]
    pub fn new(line_id: &str, statement_timestamp: &str, memo: &str, amount_cents: i64) -> Self {
        Self {
            line_id: line_id.to_owned(),
            statement_timestamp: statement_timestamp.to_owned(),
            memo: memo.to_owned(),
            amount_cents,
        }
    }

    #[must_use]
    pub fn line_id(&self) -> &str {
        &self.line_id
    }

    #[must_use]
    pub fn statement_timestamp(&self) -> &str {
        &self.statement_timestamp
    }

    #[must_use]
    pub fn memo(&self) -> &str {
        &self.memo
    }

    #[must_use]
    pub const fn amount_cents(&self) -> i64 {
        self.amount_cents
    }
}

pub trait ReconcileDataSource {
    fn fetch_reconciliation_runs(
        &self,
        month_key: Option<&str>,
        checking_account: Option<&str>,
    ) -> Vec<ReconcileRunRecord>;

    fn fetch_statement_lines_for_run(&self, run_id: &str) -> Vec<ReconcileStatementLineRecord>;
}

impl ReconcileDataSource for CliRuntime {
    fn fetch_reconciliation_runs(
        &self,
        month_key: Option<&str>,
        checking_account: Option<&str>,
    ) -> Vec<ReconcileRunRecord> {
        Self::list_reconciliation_runs(self, month_key, checking_account)
            .into_iter()
            .map(|run| {
                ReconcileRunRecord::new(
                    run.run_id(),
                    run.month_key(),
                    run.checking_account(),
                    run.variance_cents(),
                    run.reconciled(),
                    run.matched_transaction_count(),
                )
            })
            .collect()
    }

    fn fetch_statement_lines_for_run(&self, run_id: &str) -> Vec<ReconcileStatementLineRecord> {
        Self::statement_lines_for_reconciliation_run(self, run_id)
            .into_iter()
            .map(|line| {
                ReconcileStatementLineRecord::new(
                    line.line_id(),
                    line.statement_timestamp(),
                    line.memo(),
                    line.amount_cents(),
                )
            })
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct ReconcileState {
    filter_month_key: Option<String>,
    filter_checking_account: Option<String>,
    runs: Vec<ReconcileRunRecord>,
    selected_run_idx: Option<usize>,
    evidence_by_run: HashMap<String, Vec<ReconcileStatementLineRecord>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct App {
    view: View,
    exit_requested: bool,
    reconcile: ReconcileState,
}

impl Default for App {
    fn default() -> Self {
        Self {
            view: View::Home,
            exit_requested: false,
            reconcile: ReconcileState::default(),
        }
    }
}

impl App {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub const fn should_exit(&self) -> bool {
        self.exit_requested
    }

    pub const fn request_exit(&mut self) {
        self.exit_requested = true;
    }

    #[must_use]
    pub const fn view(&self) -> View {
        self.view
    }

    pub const fn set_view(&mut self, view: View) {
        self.view = view;
    }

    pub fn set_reconcile_filters(
        &mut self,
        month_key: Option<&str>,
        checking_account: Option<&str>,
    ) {
        self.reconcile.filter_month_key = month_key.map(str::to_owned);
        self.reconcile.filter_checking_account = checking_account.map(str::to_owned);
    }

    pub fn refresh_reconcile(&mut self, source: &impl ReconcileDataSource) {
        let previously_selected_run_id = self
            .selected_reconcile_run()
            .map(|run| run.run_id().to_owned());
        let runs = source.fetch_reconciliation_runs(
            self.reconcile.filter_month_key.as_deref(),
            self.reconcile.filter_checking_account.as_deref(),
        );

        let evidence_by_run = runs
            .iter()
            .map(|run| {
                (
                    run.run_id().to_owned(),
                    source.fetch_statement_lines_for_run(run.run_id()),
                )
            })
            .collect::<HashMap<_, _>>();

        let selected_run_idx = if runs.is_empty() {
            None
        } else {
            previously_selected_run_id
                .and_then(|run_id| runs.iter().position(|run| run.run_id() == run_id))
                .or(Some(0))
        };

        self.reconcile.runs = runs;
        self.reconcile.selected_run_idx = selected_run_idx;
        self.reconcile.evidence_by_run = evidence_by_run;
    }

    pub fn select_next_reconcile_run(&mut self) {
        let Some(current_idx) = self.reconcile.selected_run_idx else {
            if !self.reconcile.runs.is_empty() {
                self.reconcile.selected_run_idx = Some(0);
            }
            return;
        };

        let run_count = self.reconcile.runs.len();
        if run_count > 0 {
            self.reconcile.selected_run_idx = Some((current_idx + 1) % run_count);
        }
    }

    pub fn select_previous_reconcile_run(&mut self) {
        let Some(current_idx) = self.reconcile.selected_run_idx else {
            if !self.reconcile.runs.is_empty() {
                self.reconcile.selected_run_idx = Some(0);
            }
            return;
        };

        let run_count = self.reconcile.runs.len();
        if run_count > 0 {
            self.reconcile.selected_run_idx = Some((current_idx + run_count - 1) % run_count);
        }
    }

    pub fn handle_key(&mut self, key: char) {
        match key {
            'h' => self.set_view(View::Home),
            'b' => self.set_view(View::Budget),
            'r' => self.set_view(View::Register),
            's' => self.set_view(View::Rsu),
            'c' => self.set_view(View::Reconcile),
            'j' if self.view == View::Reconcile => self.select_next_reconcile_run(),
            'k' if self.view == View::Reconcile => self.select_previous_reconcile_run(),
            'q' => self.request_exit(),
            _ => {}
        }
    }

    #[must_use]
    pub fn render_frame(&self) -> String {
        match self.view {
            View::Home => home::render(),
            View::Budget => budget::render(),
            View::Register => register::render(),
            View::Rsu => rsu::render(),
            View::Reconcile => reconcile::render(
                self.reconcile.filter_month_key.as_deref(),
                self.reconcile.filter_checking_account.as_deref(),
                &self.reconcile.runs,
                self.selected_reconcile_run(),
                self.selected_reconcile_statement_lines(),
            ),
        }
    }

    #[must_use]
    pub fn selected_reconcile_run(&self) -> Option<&ReconcileRunRecord> {
        self.reconcile
            .selected_run_idx
            .and_then(|idx| self.reconcile.runs.get(idx))
    }

    #[must_use]
    pub fn selected_reconcile_statement_lines(&self) -> &[ReconcileStatementLineRecord] {
        let Some(run) = self.selected_reconcile_run() else {
            return &[];
        };
        self.reconcile
            .evidence_by_run
            .get(run.run_id())
            .map_or(&[], Vec::as_slice)
    }
}
