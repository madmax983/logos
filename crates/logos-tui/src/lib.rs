//! The Logos Terminal User Interface (TUI).
//!
//! This crate provides a fast, keyboard-driven interface for interacting with the
//! `logos` core library and persistent ledger. It uses a pure-function rendering
//! architecture where views are isolated from the terminal lifecycle, making them
//! easier to test and reason about. The application state is driven by an event
//! loop reacting to inputs and fetching snapshots from the runtime.

pub(crate) mod app;
pub use app::civil_from_days;
pub(crate) mod terminal;
pub(crate) mod ui;

pub use app::{
    App, AppInput, BudgetDataSource, BudgetSnapshot, HomeDataSource, HomeSnapshot,
    ReconcileDataSource, ReconcileRunRecord, ReconcileStatementLineRecord, RegisterActivityRecord,
    RegisterDataSource, RegisterSnapshot, ScopeFieldView, View,
};
pub use terminal::{
    TerminalSession, active_tab_index, render, tab_titles, view_scope_lines, view_status_lines,
};
pub use terminal::{
    key_event_to_app_char, key_event_to_app_input, read_event_input, runtime_unavailable_message,
    view_title,
};
