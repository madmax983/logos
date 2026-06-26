//! The Logos Terminal User Interface Library
//!
//! Provides a read-only terminal dashboard for visualizing the `logos` ledger.
//!
//! # Architecture
//!
//! This crate is split into three main components:
//! 1. `app`: The state machine. It manages the active view tab, cursor positions,
//!    user inputs, and caches the data snapshots loaded from the `logos-runtime`.
//! 2. `ui`: The canvas. It provides pure functions to render isolated data snapshots
//!    into styled text strings without worrying about terminal interaction.
//! 3. `terminal`: The integration layer. It manages the raw TTY session via `crossterm`
//!    and paints the `ui` strings into standard `ratatui` widgets.
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
    key_event_to_app_char, key_event_to_app_input, runtime_unavailable_message, view_title,
};
