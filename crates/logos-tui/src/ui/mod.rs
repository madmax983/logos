//! UI View Component Implementations
//!
//! # The Canvas
//!
//! This module separates the visual layout and formatting logic from the state machine defined in `app.rs`
//! and the raw terminal setup in `terminal.rs`. Each sub-module represents a distinct screen (or "View")
//! in the application, such as the dashboard (`home`), budget tracking (`budget`), transaction history (`register`),
//! and statement verification (`reconcile`).
//!
//! These components are designed as pure functions. They take isolated data snapshots (e.g., `HomeSnapshot`)
//! and render them into plain `String` outputs containing ANSI color codes (via `comfy-table` or `crossterm`).
//! This architectural choice ensures that the UI formatting can be tested independently of the heavy
//! `ratatui` terminal framework.
//!
//! ## Examples
//!
//! Rendering a view component is straightforward and requires no terminal setup:
//!
//! ```ignore
//! use logos_tui::ui::home::render;
//! use logos_tui::{HomeSnapshot, BudgetSnapshot, RegisterSnapshot};

//!
//! // 1. Create a snapshot of the data (normally provided by the App runtime)
//! let snapshot = HomeSnapshot::new(
//!     "2026-03",
//!     "assets:checking",
//!     "expenses:",
//!     5000_00,
//!     8000_00,
//!     6000_00,
//!     2000_00,
//!     Some(3000_00),
//!     Some(-1000_00)
//! );
//!
//! // 2. Render the view into a styled string
//! let output = render("2026-03", "assets:checking", "expenses:", Some(&snapshot));
//!
//! // 3. The output is a formatted UI string ready to be printed or embedded in a Ratatui widget
//! assert!(output.contains("Logos Home Dashboard"));
//! assert!(output.contains("2026-03"));
//! ```

pub mod budget;
pub mod home;
pub mod reconcile;
pub mod register;
pub mod rsu;
