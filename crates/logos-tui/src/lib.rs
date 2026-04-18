//! # The `logos-tui` Application Shell
//!
//! Welcome to the Human Interface. This module provides a rich Terminal User Interface (TUI)
//! for interacting with the `logos` accounting system.
//!
//! While `logos-cli` provides batch automation, the TUI is designed for human-driven
//! exploration. It uses `ratatui` and `crossterm` to present dashboards, registers, and
//! budgeting screens in a single interactive session.
//!
//! The architecture is built around a centralized state machine:
//! - **[`App`]**: The core orchestrator that maintains the current view, handles keystrokes,
//!   and polls the `AppRuntime` for the latest data.
//! - **Terminal Lifecycle:** Managed boundaries for alternate screen buffers and raw mode.
//!
//! ## Examples
//!
//! ```compile_fail
//! // Example usage (conceptual, requires crossterm runtime setup):
//! use logos_tui::App;
//! use logos_runtime::AppRuntime;
//! use logos_store::MemoryStore;
//! use std::path::PathBuf;
//!
//! let store = MemoryStore::default();
//! let runtime = AppRuntime::with_store(store, PathBuf::from("/tmp"), None);
//!
//! // The App takes ownership of the runtime to pull data for rendering.
//! let mut app = App::new(runtime);
//! ```

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
