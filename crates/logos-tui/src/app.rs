//! State machine and view transitions for the Logos Terminal User Interface.
//!
//! The `App` struct is the core orchestrator of the TUI. It holds the active state
//! (e.g., current view, user input buffers, loaded data snapshots) and exposes methods
//! to mutate that state based on key events.

use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};

use logos_runtime::AppRuntime;

use crate::ui::{budget, home, reconcile, register, rsu};

const DEFAULT_CHECKING_ACCOUNT: &str = "assets:checking";
const DEFAULT_EXPENSE_ACCOUNT_PREFIX: &str = "expenses:";
const DEFAULT_REGISTER_ACCOUNT: &str = "assets:checking";
const DEFAULT_REGISTER_ENTRY_LIMIT: usize = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Home,
    Budget,
    Register,
    Rsu,
    Reconcile,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppInput {
    Char(char),
    Next,
    Prev,
    NextView,
    PrevView,
    Backspace,
    Submit,
    Cancel,
    Quit,
}

use std::borrow::Cow;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopeFieldView<'a> {
    label: &'static str,
    value: Cow<'a, str>,
    focused: bool,
}

impl<'a> ScopeFieldView<'a> {
    #[must_use]
    pub fn new(label: &'static str, value: impl Into<Cow<'a, str>>, focused: bool) -> Self {
        Self {
            label,
            value: value.into(),
            focused,
        }
    }

    #[must_use]
    pub const fn label(&self) -> &'static str {
        self.label
    }

    #[must_use]
    pub fn value(&self) -> &str {
        &self.value
    }

    #[must_use]
    pub const fn focused(&self) -> bool {
        self.focused
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HomeSnapshot {
    month_key: String,
    checking_account: String,
    expense_account_prefix: String,
    checking_balance_cents: i64,
    income_cents: i64,
    expense_cents: i64,
    cashflow_cents: i64,
    budget_target_cents: Option<i64>,
    budget_variance_cents: Option<i64>,
}

impl HomeSnapshot {
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        month_key: &str,
        checking_account: &str,
        expense_account_prefix: &str,
        checking_balance_cents: i64,
        income_cents: i64,
        expense_cents: i64,
        cashflow_cents: i64,
        budget_target_cents: Option<i64>,
        budget_variance_cents: Option<i64>,
    ) -> Self {
        Self {
            month_key: month_key.to_owned(),
            checking_account: checking_account.to_owned(),
            expense_account_prefix: expense_account_prefix.to_owned(),
            checking_balance_cents,
            income_cents,
            expense_cents,
            cashflow_cents,
            budget_target_cents,
            budget_variance_cents,
        }
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
    pub fn expense_account_prefix(&self) -> &str {
        &self.expense_account_prefix
    }

    #[must_use]
    pub const fn checking_balance_cents(&self) -> i64 {
        self.checking_balance_cents
    }

    #[must_use]
    pub const fn income_cents(&self) -> i64 {
        self.income_cents
    }

    #[must_use]
    pub const fn expense_cents(&self) -> i64 {
        self.expense_cents
    }

    #[must_use]
    pub const fn cashflow_cents(&self) -> i64 {
        self.cashflow_cents
    }

    #[must_use]
    pub const fn budget_target_cents(&self) -> Option<i64> {
        self.budget_target_cents
    }

    #[must_use]
    pub const fn budget_variance_cents(&self) -> Option<i64> {
        self.budget_variance_cents
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BudgetSnapshot {
    month_key: String,
    expense_account_prefix: String,
    budget_target_cents: Option<i64>,
    actual_expense_cents: i64,
    budget_variance_cents: Option<i64>,
}

impl BudgetSnapshot {
    #[must_use]
    pub fn new(
        month_key: &str,
        expense_account_prefix: &str,
        budget_target_cents: Option<i64>,
        actual_expense_cents: i64,
        budget_variance_cents: Option<i64>,
    ) -> Self {
        Self {
            month_key: month_key.to_owned(),
            expense_account_prefix: expense_account_prefix.to_owned(),
            budget_target_cents,
            actual_expense_cents,
            budget_variance_cents,
        }
    }

    #[must_use]
    pub fn month_key(&self) -> &str {
        &self.month_key
    }

    #[must_use]
    pub fn expense_account_prefix(&self) -> &str {
        &self.expense_account_prefix
    }

    #[must_use]
    pub const fn budget_target_cents(&self) -> Option<i64> {
        self.budget_target_cents
    }

    #[must_use]
    pub const fn actual_expense_cents(&self) -> i64 {
        self.actual_expense_cents
    }

    #[must_use]
    pub const fn budget_variance_cents(&self) -> Option<i64> {
        self.budget_variance_cents
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegisterActivityRecord {
    timestamp: String,
    description: String,
    amount_cents: i64,
}

impl RegisterActivityRecord {
    #[must_use]
    pub fn new(timestamp: &str, description: &str, amount_cents: i64) -> Self {
        Self {
            timestamp: timestamp.to_owned(),
            description: description.to_owned(),
            amount_cents,
        }
    }

    #[must_use]
    pub fn timestamp(&self) -> &str {
        &self.timestamp
    }

    #[must_use]
    pub fn description(&self) -> &str {
        &self.description
    }

    #[must_use]
    pub const fn amount_cents(&self) -> i64 {
        self.amount_cents
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegisterSnapshot {
    account: String,
    balance_cents: i64,
    activity: Vec<RegisterActivityRecord>,
}

impl RegisterSnapshot {
    #[must_use]
    pub fn new(account: &str, balance_cents: i64, activity: Vec<RegisterActivityRecord>) -> Self {
        Self {
            account: account.to_owned(),
            balance_cents,
            activity,
        }
    }

    #[must_use]
    pub fn account(&self) -> &str {
        &self.account
    }

    #[must_use]
    pub const fn balance_cents(&self) -> i64 {
        self.balance_cents
    }

    #[must_use]
    pub fn activity(&self) -> &[RegisterActivityRecord] {
        &self.activity
    }
}

pub trait HomeDataSource {
    fn fetch_home_snapshot(
        &self,
        month_key: &str,
        checking_account: &str,
        expense_account_prefix: &str,
    ) -> Option<HomeSnapshot>;
}

pub trait BudgetDataSource {
    fn fetch_budget_snapshot(
        &self,
        month_key: &str,
        expense_account_prefix: &str,
    ) -> Option<BudgetSnapshot>;
}

pub trait RegisterDataSource {
    fn fetch_register_snapshot(&self, account: &str) -> Option<RegisterSnapshot>;
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

impl ReconcileDataSource for AppRuntime<logos_store_pg::PostgresStore> {
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

impl HomeDataSource for AppRuntime<logos_store_pg::PostgresStore> {
    fn fetch_home_snapshot(
        &self,
        month_key: &str,
        checking_account: &str,
        expense_account_prefix: &str,
    ) -> Option<HomeSnapshot> {
        let report = self.month_report_for(checking_account, month_key);
        let budget_target = self.budget_target_for_month(month_key, expense_account_prefix);
        let budget_variance = budget_target.map(|budget_cents| {
            self.budget_variance_for_month(month_key, budget_cents, expense_account_prefix)
        });

        Some(HomeSnapshot::new(
            month_key,
            checking_account,
            expense_account_prefix,
            report.checking_balance_cents(),
            report.income_cents(),
            report.expense_cents(),
            report.cashflow_cents(),
            budget_target,
            budget_variance,
        ))
    }
}

impl BudgetDataSource for AppRuntime<logos_store_pg::PostgresStore> {
    fn fetch_budget_snapshot(
        &self,
        month_key: &str,
        expense_account_prefix: &str,
    ) -> Option<BudgetSnapshot> {
        let budget_target = self.budget_target_for_month(month_key, expense_account_prefix);
        let budget_variance_cents = budget_target.map(|budget_cents| {
            self.budget_variance_for_month(month_key, budget_cents, expense_account_prefix)
        });
        let actual_expense_cents = if let (Some(budget_cents), Some(variance_cents)) =
            (budget_target, budget_variance_cents)
        {
            budget_cents - variance_cents
        } else {
            let now_us = current_time_us();
            self.transactions_as_of_us(now_us, now_us)
                .ok()?
                .into_iter()
                .filter(|stored| month_key_from_wallclock_utc(stored.effective_at()) == month_key)
                .flat_map(|stored| stored.transaction().postings().to_vec())
                .filter(|posting| {
                    posting
                        .account()
                        .as_str()
                        .starts_with(expense_account_prefix)
                })
                .map(|posting| posting.amount())
                .filter(|amount| *amount > 0)
                .fold(0_i64, i64::saturating_add)
        };

        Some(BudgetSnapshot::new(
            month_key,
            expense_account_prefix,
            budget_target,
            actual_expense_cents,
            budget_variance_cents,
        ))
    }
}

impl RegisterDataSource for AppRuntime<logos_store_pg::PostgresStore> {
    fn fetch_register_snapshot(&self, account: &str) -> Option<RegisterSnapshot> {
        let now_us = current_time_us();
        let transactions = self.transactions_as_of_us(now_us, now_us).ok()?;

        let mut activity = Vec::with_capacity(transactions.len());
        for stored in transactions {
            let effective_at_us = stored.effective_at();
            let mut timestamp_cache = None;
            for posting in stored.transaction().postings() {
                if posting.account().as_str() == account {
                    let timestamp = timestamp_cache
                        .get_or_insert_with(|| date_string_from_wallclock_utc(effective_at_us));
                    activity.push((
                        effective_at_us,
                        RegisterActivityRecord::new(
                            timestamp,
                            stored.transaction().description(),
                            posting.amount(),
                        ),
                    ));
                }
            }
        }

        activity.sort_by(|left, right| right.0.cmp(&left.0));
        let activity = activity
            .into_iter()
            .take(DEFAULT_REGISTER_ENTRY_LIMIT)
            .map(|(_, record)| record)
            .collect();

        Some(RegisterSnapshot::new(
            account,
            self.register_balance_for(account),
            activity,
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct HomeState {
    month_key: String,
    checking_account: String,
    expense_account_prefix: String,
    snapshot: Option<HomeSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct BudgetState {
    month_key: String,
    expense_account_prefix: String,
    snapshot: Option<BudgetSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct RegisterState {
    account: String,
    snapshot: Option<RegisterSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct ReconcileState {
    filter_month_key: Option<String>,
    filter_checking_account: Option<String>,
    runs: Vec<ReconcileRunRecord>,
    selected_run_idx: Option<usize>,
    evidence_by_run: HashMap<String, Vec<ReconcileStatementLineRecord>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScopeFieldKey {
    HomeMonthKey,
    HomeCheckingAccount,
    HomeExpenseAccountPrefix,
    BudgetMonthKey,
    BudgetExpenseAccountPrefix,
    RegisterAccount,
    ReconcileMonthFilter,
    ReconcileCheckingAccountFilter,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ScopeDraftField {
    key: ScopeFieldKey,
    label: &'static str,
    value: String,
}

impl ScopeDraftField {
    fn new(key: ScopeFieldKey, label: &'static str, value: impl Into<String>) -> Self {
        Self {
            key,
            label,
            value: value.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ScopeEditorState {
    fields: Vec<ScopeDraftField>,
    focused_field_idx: usize,
}

impl ScopeEditorState {
    const fn new(fields: Vec<ScopeDraftField>) -> Self {
        Self {
            fields,
            focused_field_idx: 0,
        }
    }

    fn focused_field_mut(&mut self) -> Option<&mut ScopeDraftField> {
        self.fields.get_mut(self.focused_field_idx)
    }

    fn select_next_field(&mut self) {
        let field_count = self.fields.len();
        if field_count > 0 {
            self.focused_field_idx = (self.focused_field_idx + 1) % field_count;
        }
    }

    fn select_previous_field(&mut self) {
        let field_count = self.fields.len();
        if field_count > 0 {
            self.focused_field_idx = (self.focused_field_idx + field_count - 1) % field_count;
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct App {
    view: View,
    exit_requested: bool,
    home: HomeState,
    budget: BudgetState,
    register: RegisterState,
    reconcile: ReconcileState,
    scope_editor: Option<ScopeEditorState>,
    scope_error: Option<String>,
}

impl Default for App {
    fn default() -> Self {
        let current_month_key =
            logos_runtime::AppRuntime::<logos_store_pg::PostgresStore>::current_month_key_local();
        Self {
            view: View::Home,
            exit_requested: false,
            home: HomeState {
                month_key: current_month_key.clone(),
                checking_account: DEFAULT_CHECKING_ACCOUNT.to_owned(),
                expense_account_prefix: DEFAULT_EXPENSE_ACCOUNT_PREFIX.to_owned(),
                snapshot: None,
            },
            budget: BudgetState {
                month_key: current_month_key,
                expense_account_prefix: DEFAULT_EXPENSE_ACCOUNT_PREFIX.to_owned(),
                snapshot: None,
            },
            register: RegisterState {
                account: DEFAULT_REGISTER_ACCOUNT.to_owned(),
                snapshot: None,
            },
            reconcile: ReconcileState::default(),
            scope_editor: None,
            scope_error: None,
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

    #[must_use]
    pub fn home_month_key(&self) -> &str {
        &self.home.month_key
    }

    #[must_use]
    pub fn home_checking_account(&self) -> &str {
        &self.home.checking_account
    }

    #[must_use]
    pub fn home_expense_account_prefix(&self) -> &str {
        &self.home.expense_account_prefix
    }

    #[must_use]
    pub const fn home_snapshot(&self) -> Option<&HomeSnapshot> {
        self.home.snapshot.as_ref()
    }

    #[must_use]
    pub fn budget_month_key(&self) -> &str {
        &self.budget.month_key
    }

    #[must_use]
    pub fn budget_expense_account_prefix(&self) -> &str {
        &self.budget.expense_account_prefix
    }

    #[must_use]
    pub const fn budget_snapshot(&self) -> Option<&BudgetSnapshot> {
        self.budget.snapshot.as_ref()
    }

    #[must_use]
    pub fn register_account(&self) -> &str {
        &self.register.account
    }

    #[must_use]
    pub const fn register_snapshot(&self) -> Option<&RegisterSnapshot> {
        self.register.snapshot.as_ref()
    }

    #[must_use]
    pub fn reconcile_filter_month_key(&self) -> Option<&str> {
        self.reconcile.filter_month_key.as_deref()
    }

    #[must_use]
    pub fn reconcile_filter_checking_account(&self) -> Option<&str> {
        self.reconcile.filter_checking_account.as_deref()
    }

    #[must_use]
    pub const fn is_scope_editing(&self) -> bool {
        self.scope_editor.is_some()
    }

    #[must_use]
    pub fn scope_error_message(&self) -> Option<&str> {
        self.scope_error.as_deref()
    }

    #[must_use]
    fn home_scope_fields(&self) -> Vec<ScopeFieldView<'_>> {
        vec![
            ScopeFieldView::new(
                "Month",
                self.home.snapshot.as_ref().map_or_else(
                    || Cow::Borrowed(self.home.month_key.as_str()),
                    |snapshot| Cow::Borrowed(snapshot.month_key()),
                ),
                false,
            ),
            ScopeFieldView::new(
                "Checking",
                self.home.snapshot.as_ref().map_or_else(
                    || Cow::Borrowed(self.home.checking_account.as_str()),
                    |snapshot| Cow::Borrowed(snapshot.checking_account()),
                ),
                false,
            ),
            ScopeFieldView::new(
                "Expenses",
                self.home.snapshot.as_ref().map_or_else(
                    || Cow::Borrowed(self.home.expense_account_prefix.as_str()),
                    |snapshot| Cow::Borrowed(snapshot.expense_account_prefix()),
                ),
                false,
            ),
        ]
    }

    fn budget_scope_fields(&self) -> Vec<ScopeFieldView<'_>> {
        vec![
            ScopeFieldView::new(
                "Month",
                self.budget.snapshot.as_ref().map_or_else(
                    || Cow::Borrowed(self.budget.month_key.as_str()),
                    |snapshot| Cow::Borrowed(snapshot.month_key()),
                ),
                false,
            ),
            ScopeFieldView::new(
                "Expenses",
                self.budget.snapshot.as_ref().map_or_else(
                    || Cow::Borrowed(self.budget.expense_account_prefix.as_str()),
                    |snapshot| Cow::Borrowed(snapshot.expense_account_prefix()),
                ),
                false,
            ),
        ]
    }

    fn register_scope_fields(&self) -> Vec<ScopeFieldView<'_>> {
        vec![ScopeFieldView::new(
            "Account",
            self.register.snapshot.as_ref().map_or_else(
                || Cow::Borrowed(self.register.account.as_str()),
                |snapshot| Cow::Borrowed(snapshot.account()),
            ),
            false,
        )]
    }

    fn reconcile_scope_fields(&self) -> Vec<ScopeFieldView<'_>> {
        vec![
            ScopeFieldView::new(
                "Month Filter",
                self.reconcile
                    .filter_month_key
                    .as_deref()
                    .map_or_else(|| Cow::Borrowed("*"), Cow::Borrowed),
                false,
            ),
            ScopeFieldView::new(
                "Account Filter",
                self.reconcile
                    .filter_checking_account
                    .as_deref()
                    .map_or_else(|| Cow::Borrowed("*"), Cow::Borrowed),
                false,
            ),
        ]
    }

    #[must_use]
    pub fn scope_field_views(&self) -> Vec<ScopeFieldView<'_>> {
        if let Some(editor) = &self.scope_editor {
            return editor
                .fields
                .iter()
                .enumerate()
                .map(|(idx, field)| {
                    ScopeFieldView::new(
                        field.label,
                        Cow::Borrowed(field.value.as_str()),
                        idx == editor.focused_field_idx,
                    )
                })
                .collect();
        }

        match self.view {
            View::Home => self.home_scope_fields(),
            View::Budget => self.budget_scope_fields(),
            View::Register => self.register_scope_fields(),
            View::Rsu => vec![ScopeFieldView::new(
                "Forecast scope",
                Cow::Borrowed("pending"),
                false,
            )],
            View::Reconcile => self.reconcile_scope_fields(),
        }
    }

    /// Transitions the application to a new primary view tab.
    ///
    /// Clears any active input editors or error states.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use logos_tui::{App, View};
    ///
    /// let mut app = App::new();
    /// app.set_view(View::Register);
    /// assert_eq!(app.view(), View::Register);
    /// ```
    pub fn set_view(&mut self, view: View) {
        self.scope_editor = None;
        self.scope_error = None;
        self.view = view;
    }

    /// Reloads data for the Home (Dashboard) view from the provided source.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use logos_tui::{App, HomeDataSource, HomeSnapshot};
    ///
    /// struct MockSource;
    /// impl HomeDataSource for MockSource {
    ///     fn fetch_home_snapshot(&self, _month: &str, _chk: &str, _exp: &str) -> Option<HomeSnapshot> {
    ///         None
    ///     }
    /// }
    ///
    /// let mut app = App::new();
    /// app.refresh_home(&MockSource);
    /// ```
    pub fn refresh_home(&mut self, source: &impl HomeDataSource) {
        self.home.snapshot = source.fetch_home_snapshot(
            &self.home.month_key,
            &self.home.checking_account,
            &self.home.expense_account_prefix,
        );
    }

    /// Reloads envelope allocations and actual spending for the Budget view.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use logos_tui::{App, BudgetDataSource, BudgetSnapshot};
    ///
    /// struct MockSource;
    /// impl BudgetDataSource for MockSource {
    ///     fn fetch_budget_snapshot(&self, _month: &str, _exp: &str) -> Option<BudgetSnapshot> { None }
    /// }
    ///
    /// let mut app = App::new();
    /// app.refresh_budget(&MockSource);
    /// ```
    pub fn refresh_budget(&mut self, source: &impl BudgetDataSource) {
        self.budget.snapshot = source
            .fetch_budget_snapshot(&self.budget.month_key, &self.budget.expense_account_prefix);
    }

    /// Reloads the chronological transaction list for the active Register account.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use logos_tui::{App, RegisterDataSource, RegisterSnapshot};
    ///
    /// struct MockSource;
    /// impl RegisterDataSource for MockSource {
    ///     fn fetch_register_snapshot(&self, _account: &str) -> Option<RegisterSnapshot> { None }
    /// }
    ///
    /// let mut app = App::new();
    /// app.refresh_register(&MockSource);
    /// ```
    pub fn refresh_register(&mut self, source: &impl RegisterDataSource) {
        self.register.snapshot = source.fetch_register_snapshot(&self.register.account);
    }

    /// Adjusts the global filtering criteria applied to the Reconcile history list.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use logos_tui::App;
    ///
    /// let mut app = App::new();
    /// app.set_reconcile_filters(Some("2026-03"), None);
    /// assert_eq!(app.reconcile_filter_month_key(), Some("2026-03"));
    /// ```
    pub fn set_reconcile_filters(
        &mut self,
        month_key: Option<&str>,
        checking_account: Option<&str>,
    ) {
        self.reconcile.filter_month_key = month_key.map(str::to_owned);
        self.reconcile.filter_checking_account = checking_account.map(str::to_owned);
    }

    /// Re-fetches all matching reconciliation runs and their associated statement evidence.
    ///
    /// Maintains the current selection index if the previously selected run is still
    /// present in the newly fetched dataset.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use logos_tui::{App, ReconcileDataSource, ReconcileRunRecord, ReconcileStatementLineRecord};
    ///
    /// struct MockSource;
    /// impl ReconcileDataSource for MockSource {
    ///     fn fetch_reconciliation_runs(&self, _: Option<&str>, _: Option<&str>) -> Vec<ReconcileRunRecord> { vec![] }
    ///     fn fetch_statement_lines_for_run(&self, _: &str) -> Vec<ReconcileStatementLineRecord> { vec![] }
    /// }
    ///
    /// let mut app = App::new();
    /// app.refresh_reconcile(&MockSource);
    /// ```
    pub fn refresh_reconcile(&mut self, source: &impl ReconcileDataSource) {
        let previously_selected_run_id = self
            .selected_reconcile_run()
            .map(|run| run.run_id().to_owned());
        let runs = source.fetch_reconciliation_runs(
            self.reconcile.filter_month_key.as_deref(),
            self.reconcile.filter_checking_account.as_deref(),
        );

        let mut evidence_by_run = HashMap::with_capacity(runs.len());
        for run in &runs {
            evidence_by_run.insert(
                run.run_id().to_owned(),
                source.fetch_statement_lines_for_run(run.run_id()),
            );
        }

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

    /// Dispatches a refresh call to the data source based on the currently active view tab.
    ///
    /// This prevents the application from eagerly fetching expensive projections for
    /// tabs the user isn't even looking at.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use logos_tui::{App, HomeDataSource, HomeSnapshot, BudgetDataSource, BudgetSnapshot, RegisterDataSource, RegisterSnapshot, ReconcileDataSource, ReconcileRunRecord, ReconcileStatementLineRecord};
    ///
    /// struct MockSource;
    /// impl HomeDataSource for MockSource { fn fetch_home_snapshot(&self, _m: &str, _c: &str, _e: &str) -> Option<HomeSnapshot> { None } }
    /// impl BudgetDataSource for MockSource { fn fetch_budget_snapshot(&self, _m: &str, _e: &str) -> Option<BudgetSnapshot> { None } }
    /// impl RegisterDataSource for MockSource { fn fetch_register_snapshot(&self, _a: &str) -> Option<RegisterSnapshot> { None } }
    /// impl ReconcileDataSource for MockSource {
    ///     fn fetch_reconciliation_runs(&self, _: Option<&str>, _: Option<&str>) -> Vec<ReconcileRunRecord> { vec![] }
    ///     fn fetch_statement_lines_for_run(&self, _: &str) -> Vec<ReconcileStatementLineRecord> { vec![] }
    /// }
    ///
    /// let mut app = App::new();
    /// app.refresh_current_view(&MockSource);
    /// ```
    pub fn refresh_current_view(
        &mut self,
        source: &(impl HomeDataSource + BudgetDataSource + RegisterDataSource + ReconcileDataSource),
    ) {
        match self.view {
            View::Home => self.refresh_home(source),
            View::Budget => self.refresh_budget(source),
            View::Register => self.refresh_register(source),
            View::Rsu => {}
            View::Reconcile => self.refresh_reconcile(source),
        }
    }

    /// Moves the cursor down one row in the Reconcile view's run list.
    ///
    /// Wraps around to the top if the cursor is at the bottom.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use logos_tui::App;
    /// let mut app = App::new();
    /// app.select_next_reconcile_run(); // Safe to call on empty lists
    /// ```
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

    /// Moves the cursor up one row in the Reconcile view's run list.
    ///
    /// Wraps around to the bottom if the cursor is at the top.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use logos_tui::App;
    /// let mut app = App::new();
    /// app.select_previous_reconcile_run(); // Safe to call on empty lists
    /// ```
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

    /// Processes a single semantic application input event.
    ///
    /// This routes the input based on current mode (e.g. Scope Editor vs Normal Navigation)
    /// and mutates the application state accordingly.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use logos_tui::{App, AppInput};
    /// let mut app = App::new();
    /// app.handle_input(AppInput::Quit);
    /// assert!(app.should_exit());
    /// ```
    pub fn handle_input(&mut self, input: AppInput) {
        if input == AppInput::Quit {
            self.request_exit();
            return;
        }

        if self.is_scope_editing() {
            self.handle_scope_editor_input(input);
            return;
        }

        match input {
            AppInput::Char('h') => self.set_view(View::Home),
            AppInput::Char('b') => self.set_view(View::Budget),
            AppInput::Char('r') => self.set_view(View::Register),
            AppInput::Char('s') => self.set_view(View::Rsu),
            AppInput::Char('c') => self.set_view(View::Reconcile),
            AppInput::Char('i') => self.enter_scope_editor(),
            AppInput::NextView => self.select_next_view(),
            AppInput::PrevView => self.select_previous_view(),
            AppInput::Char('j') | AppInput::Next if self.view == View::Reconcile => {
                self.select_next_reconcile_run();
            }
            AppInput::Char('k') | AppInput::Prev if self.view == View::Reconcile => {
                self.select_previous_reconcile_run();
            }
            AppInput::Char('q') => self.request_exit(),
            AppInput::Cancel
            | AppInput::Backspace
            | AppInput::Submit
            | AppInput::Next
            | AppInput::Prev
            | AppInput::Quit
            | AppInput::Char(_) => {}
        }
    }

    /// Convenience wrapper for processing raw character inputs.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use logos_tui::{App, View};
    /// let mut app = App::new();
    /// app.handle_key('b');
    /// assert_eq!(app.view(), View::Budget);
    /// ```
    pub fn handle_key(&mut self, key: char) {
        self.handle_input(AppInput::Char(key));
    }

    #[must_use]
    pub fn render_frame(&self) -> String {
        match self.view {
            View::Home => home::render(
                &self.home.month_key,
                &self.home.checking_account,
                &self.home.expense_account_prefix,
                self.home.snapshot.as_ref(),
            ),
            View::Budget => budget::render(
                &self.budget.month_key,
                &self.budget.expense_account_prefix,
                self.budget.snapshot.as_ref(),
            ),
            View::Register => {
                register::render(&self.register.account, self.register.snapshot.as_ref())
            }
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

    fn enter_scope_editor(&mut self) {
        self.scope_error = None;
        let fields = match self.view {
            View::Home => vec![
                ScopeDraftField::new(
                    ScopeFieldKey::HomeMonthKey,
                    "Month",
                    self.home.month_key.clone(),
                ),
                ScopeDraftField::new(
                    ScopeFieldKey::HomeCheckingAccount,
                    "Checking",
                    self.home.checking_account.clone(),
                ),
                ScopeDraftField::new(
                    ScopeFieldKey::HomeExpenseAccountPrefix,
                    "Expenses",
                    self.home.expense_account_prefix.clone(),
                ),
            ],
            View::Budget => vec![
                ScopeDraftField::new(
                    ScopeFieldKey::BudgetMonthKey,
                    "Month",
                    self.budget.month_key.clone(),
                ),
                ScopeDraftField::new(
                    ScopeFieldKey::BudgetExpenseAccountPrefix,
                    "Expenses",
                    self.budget.expense_account_prefix.clone(),
                ),
            ],
            View::Register => vec![ScopeDraftField::new(
                ScopeFieldKey::RegisterAccount,
                "Account",
                self.register.account.clone(),
            )],
            View::Rsu => Vec::new(),
            View::Reconcile => vec![
                ScopeDraftField::new(
                    ScopeFieldKey::ReconcileMonthFilter,
                    "Month Filter",
                    self.reconcile.filter_month_key.clone().unwrap_or_default(),
                ),
                ScopeDraftField::new(
                    ScopeFieldKey::ReconcileCheckingAccountFilter,
                    "Account Filter",
                    self.reconcile
                        .filter_checking_account
                        .clone()
                        .unwrap_or_default(),
                ),
            ],
        };

        if !fields.is_empty() {
            self.scope_editor = Some(ScopeEditorState::new(fields));
        }
    }

    fn handle_scope_editor_input(&mut self, input: AppInput) {
        match input {
            AppInput::Char(ch) if !ch.is_control() => {
                self.scope_error = None;
                if let Some(field) = self
                    .scope_editor
                    .as_mut()
                    .and_then(ScopeEditorState::focused_field_mut)
                {
                    field.value.push(ch);
                }
            }
            AppInput::Next => {
                self.scope_error = None;
                if let Some(editor) = &mut self.scope_editor {
                    editor.select_next_field();
                }
            }
            AppInput::Prev => {
                self.scope_error = None;
                if let Some(editor) = &mut self.scope_editor {
                    editor.select_previous_field();
                }
            }
            AppInput::Backspace => {
                self.scope_error = None;
                if let Some(field) = self
                    .scope_editor
                    .as_mut()
                    .and_then(ScopeEditorState::focused_field_mut)
                {
                    field.value.pop();
                }
            }
            AppInput::Submit => self.apply_scope_editor(),
            AppInput::Cancel => {
                self.scope_editor = None;
                self.scope_error = None;
            }
            AppInput::NextView | AppInput::PrevView | AppInput::Quit | AppInput::Char(_) => {}
        }
    }

    fn apply_scope_editor(&mut self) {
        let Some(editor) = self.scope_editor.take() else {
            return;
        };

        let normalized_fields = match normalized_scope_changes(&editor) {
            Ok(fields) => fields,
            Err(message) => {
                self.scope_error = Some(message);
                self.scope_editor = Some(editor);
                return;
            }
        };

        self.scope_error = None;
        for (key, value) in normalized_fields {
            match (key, value) {
                (ScopeFieldKey::HomeMonthKey, Some(value)) => {
                    self.home.month_key = value;
                    self.home.snapshot = None;
                }
                (ScopeFieldKey::HomeCheckingAccount, Some(value)) => {
                    self.home.checking_account = value;
                    self.home.snapshot = None;
                }
                (ScopeFieldKey::HomeExpenseAccountPrefix, Some(value)) => {
                    self.home.expense_account_prefix = value;
                    self.home.snapshot = None;
                }
                (ScopeFieldKey::BudgetMonthKey, Some(value)) => {
                    self.budget.month_key = value;
                    self.budget.snapshot = None;
                }
                (ScopeFieldKey::BudgetExpenseAccountPrefix, Some(value)) => {
                    self.budget.expense_account_prefix = value;
                    self.budget.snapshot = None;
                }
                (ScopeFieldKey::RegisterAccount, Some(value)) => {
                    self.register.account = value;
                    self.register.snapshot = None;
                }
                (ScopeFieldKey::ReconcileMonthFilter, value) => {
                    self.reconcile.filter_month_key = value;
                    self.clear_reconcile_results();
                }
                (ScopeFieldKey::ReconcileCheckingAccountFilter, value) => {
                    self.reconcile.filter_checking_account = value;
                    self.clear_reconcile_results();
                }
                _ => {}
            }
        }
    }

    fn clear_reconcile_results(&mut self) {
        self.reconcile.runs.clear();
        self.reconcile.selected_run_idx = None;
        self.reconcile.evidence_by_run.clear();
    }

    fn select_next_view(&mut self) {
        let next = match self.view {
            View::Home => View::Budget,
            View::Budget => View::Register,
            View::Register => View::Rsu,
            View::Rsu => View::Reconcile,
            View::Reconcile => View::Home,
        };
        self.set_view(next);
    }

    fn select_previous_view(&mut self) {
        let previous = match self.view {
            View::Home => View::Reconcile,
            View::Budget => View::Home,
            View::Register => View::Budget,
            View::Rsu => View::Register,
            View::Reconcile => View::Rsu,
        };
        self.set_view(previous);
    }
}

fn normalized_scope_changes(
    editor: &ScopeEditorState,
) -> Result<Vec<(ScopeFieldKey, Option<String>)>, String> {
    editor
        .fields
        .iter()
        .map(|field| {
            let normalized = match field.key {
                ScopeFieldKey::HomeMonthKey | ScopeFieldKey::BudgetMonthKey => {
                    Some(normalize_required_month(&field.value, field.label)?)
                }
                ScopeFieldKey::HomeCheckingAccount
                | ScopeFieldKey::HomeExpenseAccountPrefix
                | ScopeFieldKey::BudgetExpenseAccountPrefix
                | ScopeFieldKey::RegisterAccount => {
                    Some(normalize_required_scope(&field.value, field.label)?)
                }
                ScopeFieldKey::ReconcileMonthFilter => {
                    normalize_optional_month(&field.value, field.label)?
                }
                ScopeFieldKey::ReconcileCheckingAccountFilter => {
                    normalize_optional_scope(&field.value, field.label)?
                }
            };
            Ok((field.key, normalized))
        })
        .collect()
}

fn normalize_required_scope(value: &str, label: &str) -> Result<String, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(format!("{label} cannot be blank"));
    }

    if trimmed.chars().any(char::is_whitespace) {
        return Err(format!("{label} cannot contain spaces"));
    }

    Ok(trimmed.to_owned())
}

fn normalize_optional_scope(value: &str, label: &str) -> Result<Option<String>, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }

    if trimmed.chars().any(char::is_whitespace) {
        return Err(format!("{label} cannot contain spaces"));
    }

    Ok(Some(trimmed.to_owned()))
}

fn normalize_required_month(value: &str, label: &str) -> Result<String, String> {
    let trimmed = value.trim();
    if is_valid_month_key(trimmed) {
        Ok(trimmed.to_owned())
    } else {
        Err(format!("{label} must use YYYY-MM"))
    }
}

fn normalize_optional_month(value: &str, label: &str) -> Result<Option<String>, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }

    if is_valid_month_key(trimmed) {
        Ok(Some(trimmed.to_owned()))
    } else {
        Err(format!("{label} must use YYYY-MM"))
    }
}

fn is_valid_month_key(value: &str) -> bool {
    let mut parts = value.split('-');
    let Some(year) = parts.next() else {
        return false;
    };
    let Some(month) = parts.next() else {
        return false;
    };

    if parts.next().is_some() || year.len() != 4 || month.len() != 2 {
        return false;
    }

    if !year.chars().all(|ch| ch.is_ascii_digit()) || !month.chars().all(|ch| ch.is_ascii_digit()) {
        return false;
    }

    matches!(month.parse::<u8>(), Ok(1..=12))
}

fn current_time_us() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_micros())
        .ok()
        .and_then(|micros| i64::try_from(micros).ok())
        .unwrap_or(0)
}

fn month_key_from_wallclock_utc(wallclock_us: i64) -> String {
    let secs = wallclock_us.div_euclid(1_000_000);
    let days = secs.div_euclid(86_400);
    let (year, month, _) = civil_from_days(days);
    format!("{year:04}-{month:02}")
}

fn date_string_from_wallclock_utc(wallclock_us: i64) -> String {
    let secs = wallclock_us.div_euclid(1_000_000);
    let days = secs.div_euclid(86_400);
    let seconds_of_day = secs.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    let hour = seconds_of_day.div_euclid(3_600);
    let minute = seconds_of_day.rem_euclid(3_600).div_euclid(60);
    let second = seconds_of_day.rem_euclid(60);
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}")
}

#[must_use]
/// Converts days since the UNIX epoch into a civil date (year, month, day).
///
/// This is used heavily for resolving timestamps into specific month buckets
/// for rendering historical activity without relying on heavy timezone crates.
///
/// ## Examples
///
/// ```
/// use logos_tui::civil_from_days;
///
/// // Day 0 is 1970-01-01
/// assert_eq!(civil_from_days(0), (1970, 1, 1));
/// ```
pub fn civil_from_days(days_since_unix_epoch: i64) -> (i64, u32, u32) {
    let z = days_since_unix_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let day_of_era = z - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let mut year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    if month <= 2 {
        year += 1;
    }

    let month_u32 = u32::try_from(month).unwrap_or(1);
    let day_u32 = u32::try_from(day).unwrap_or(1);
    (year, month_u32, day_u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_civil_from_days_boundaries() {
        // Known dates around month boundaries
        // Day 0 = Jan 1, 1970
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        assert_eq!(civil_from_days(31), (1970, 2, 1));
        assert_eq!(civil_from_days(59), (1970, 3, 1)); // 1970 is not a leap year (28 days in feb)

        // Day 146_096 = Dec 31, 2369
        assert_eq!(civil_from_days(146_096), (2369, 12, 31));

        // Day -1 = Dec 31, 1969
        assert_eq!(civil_from_days(-1), (1969, 12, 31));

        // Test logic branch `z >= 0` vs `z < 0` where z = days + 719_468.
        // z = -1 implies days = -719_469
        assert_eq!(civil_from_days(-719_469), (0, 2, 29));
        // z = 0 implies days = -719_468
        assert_eq!(civil_from_days(-719_468), (0, 3, 1));

        // Leap year boundary test (2000 was leap year)
        // 2000-02-29 is days = 10957 + 28
        // 2000-01-01 is days = 10957
        assert_eq!(civil_from_days(10957), (2000, 1, 1));
        assert_eq!(civil_from_days(10957 + 31 + 28), (2000, 2, 29));
        assert_eq!(civil_from_days(10957 + 31 + 29), (2000, 3, 1));
    }
}
