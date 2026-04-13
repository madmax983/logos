#![allow(missing_docs)]
pub mod app;
pub mod terminal;
pub mod ui;

pub use app::{
    App, AppInput, BudgetDataSource, BudgetSnapshot, HomeDataSource, HomeSnapshot,
    ReconcileDataSource, ReconcileRunRecord, ReconcileStatementLineRecord, RegisterActivityRecord,
    RegisterDataSource, RegisterSnapshot, ScopeFieldView, View,
};
pub use terminal::{active_tab_index, tab_titles, view_scope_lines, view_status_lines};
pub use terminal::{
    key_event_to_app_char, key_event_to_app_input, runtime_unavailable_message, view_title,
};
