//! Terminal session lifecycle and raw rendering abstractions.
//!
//! Provides the boundary between the internal `App` state and the `ratatui` + `crossterm`
//! UI primitives. It manages the alternate screen, raw mode, cursor visibility,
//! and input event polling.

use std::{io, time::Duration};

use crossterm::{
    cursor::{Hide, Show},
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph, Tabs, Wrap},
};

use crate::{App, AppInput, View};
use logos_cli::format::currency;

const EVENT_POLL_INTERVAL: Duration = Duration::from_millis(250);

/// A managed wrapper around the TTY for rendering and input handling.
///
/// Ensures the terminal is correctly restored to its original state (cursor visible,
/// raw mode disabled) when the TUI application exits.
///
/// ## Examples
///
/// ```rust,no_run
/// use logos_tui::TerminalSession;
///
/// // Automatically enters alternate screen and raw mode.
/// let mut session = TerminalSession::enter().unwrap();
/// ```
pub struct TerminalSession {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
}

impl TerminalSession {
    /// Enters raw mode, switches to the alternate screen, and creates a terminal backend.
    ///
    /// # Errors
    ///
    /// Returns an error when terminal raw mode, alternate-screen setup, or terminal creation fails.
    pub fn enter() -> io::Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, Hide)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        Ok(Self { terminal })
    }

    /// Draws the current application frame.
    ///
    /// # Errors
    ///
    /// Returns an error when the terminal backend draw operation fails.
    pub fn draw(&mut self, app: &App, runtime_available: bool) -> io::Result<()> {
        self.terminal
            .draw(|frame| render(frame, app, runtime_available))
            .map(|_| ())
    }

    /// Polls for the next terminal event and translates supported key presses into app inputs.
    ///
    /// # Errors
    ///
    /// Returns an error when terminal event polling or reading fails.
    pub fn next_input(&mut self) -> io::Result<Option<AppInput>> {
        if !event::poll(EVENT_POLL_INTERVAL)? {
            return Ok(None);
        }

        let event = event::read()?;
        Ok(read_event_input(&event))
    }

    /// Polls for the next terminal event and translates supported key presses into app chars.
    ///
    /// # Errors
    ///
    /// Returns an error when terminal event polling or reading fails.
    pub fn next_key_char(&mut self) -> io::Result<Option<char>> {
        Ok(self.next_input()?.and_then(app_input_to_char))
    }

    fn restore(&mut self) -> io::Result<()> {
        disable_raw_mode()?;
        execute!(self.terminal.backend_mut(), LeaveAlternateScreen, Show)?;
        self.terminal.show_cursor()?;
        Ok(())
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        let _ = self.restore();
    }
}

/// Draws the primary application layout within the provided terminal frame.
///
/// Takes the internal state variables from `App` and paints the Ratatui
/// borders, tabs, scope panel, and the main view body.
///
/// ## Examples
///
/// ```rust,no_run
/// use ratatui::{backend::TestBackend, Terminal};
/// use logos_tui::App;
/// use logos_tui::render;
///
/// let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
/// let app = App::new();
///
/// terminal.draw(|frame| render(frame, &app, true)).unwrap();
/// ```
pub fn render(frame: &mut Frame<'_>, app: &App, runtime_available: bool) {
    let scope_lines = view_scope_lines(app);
    let status_lines = view_status_lines(app, runtime_available);
    let footer_lines = footer_lines(app);
    let layout = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(3),
        Constraint::Length(
            u16::try_from(scope_lines.len().max(status_lines.len()).saturating_add(2)).unwrap_or(4),
        ),
        Constraint::Min(1),
        Constraint::Length(u16::try_from(footer_lines.len()).unwrap_or(2)),
    ])
    .split(frame.area());

    let header = Paragraph::new(Line::from(format!(
        "Logos TUI | View: {}",
        view_title(app.view())
    )))
    .style(Style::default().add_modifier(Modifier::BOLD));

    let tabs = Tabs::new(tab_titles())
        .block(Block::bordered().title("Views"))
        .select(active_tab_index(app.view()))
        .highlight_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );

    let status_layout =
        Layout::horizontal([Constraint::Percentage(55), Constraint::Percentage(45)])
            .split(layout[2]);
    let scope_widget = Paragraph::new(scope_lines.join("\n"))
        .block(Block::bordered().title("Scope"))
        .wrap(Wrap { trim: false });
    let status_widget = Paragraph::new(status_lines)
        .block(Block::bordered().title("Status"))
        .wrap(Wrap { trim: false });

    let body = Paragraph::new(app.render_frame())
        .block(Block::bordered().title(view_title(app.view())))
        .wrap(Wrap { trim: false });

    let footer = Paragraph::new(footer_lines).wrap(Wrap { trim: false });

    frame.render_widget(header, layout[0]);
    frame.render_widget(tabs, layout[1]);
    frame.render_widget(scope_widget, status_layout[0]);
    frame.render_widget(status_widget, status_layout[1]);
    frame.render_widget(body, layout[3]);
    frame.render_widget(footer, layout[4]);
}

#[must_use]
pub fn read_event_input(event: &Event) -> Option<AppInput> {
    match event {
        Event::Key(key_event) => key_event_to_app_input(*key_event),
        _ => None,
    }
}

#[must_use]
#[allow(clippy::missing_const_for_fn)]
pub fn key_event_to_app_input(key_event: KeyEvent) -> Option<AppInput> {
    if !matches!(key_event.kind, KeyEventKind::Press | KeyEventKind::Repeat) {
        return None;
    }

    match key_event.code {
        KeyCode::Esc => Some(AppInput::Cancel),
        KeyCode::Left => Some(AppInput::PrevView),
        KeyCode::Right => Some(AppInput::NextView),
        KeyCode::Up | KeyCode::BackTab => Some(AppInput::Prev),
        KeyCode::Down | KeyCode::Tab => Some(AppInput::Next),
        KeyCode::Backspace => Some(AppInput::Backspace),
        KeyCode::Enter => Some(AppInput::Submit),
        KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(AppInput::Quit)
        }
        KeyCode::Char(ch) => Some(AppInput::Char(ch)),
        _ => None,
    }
}

#[must_use]
pub fn key_event_to_app_char(key_event: KeyEvent) -> Option<char> {
    key_event_to_app_input(key_event).and_then(app_input_to_char)
}

#[must_use]
const fn app_input_to_char(input: AppInput) -> Option<char> {
    match input {
        AppInput::Char(ch) => Some(ch),
        AppInput::Next => Some('j'),
        AppInput::Prev => Some('k'),
        AppInput::Quit => Some('q'),
        AppInput::Backspace
        | AppInput::Submit
        | AppInput::Cancel
        | AppInput::NextView
        | AppInput::PrevView => None,
    }
}

#[must_use]
pub const fn tab_titles() -> [&'static str; 5] {
    ["Home", "Budget", "Register", "RSU", "Reconcile"]
}

#[must_use]
pub const fn active_tab_index(view: View) -> usize {
    match view {
        View::Home => 0,
        View::Budget => 1,
        View::Register => 2,
        View::Rsu => 3,
        View::Reconcile => 4,
    }
}

#[must_use]
pub const fn view_title(view: View) -> &'static str {
    match view {
        View::Home => "Home",
        View::Budget => "Budget",
        View::Register => "Register",
        View::Rsu => "RSU",
        View::Reconcile => "Reconcile",
    }
}

#[must_use]
pub fn view_scope_lines(app: &App) -> Vec<String> {
    let editing = app.is_scope_editing();
    app.scope_field_views()
        .into_iter()
        .map(|field| {
            let prefix = if field.focused() {
                "> "
            } else if editing {
                "  "
            } else {
                ""
            };
            format!("{prefix}{}: {}", field.label(), field.value())
        })
        .collect()
}

#[must_use]
pub const fn runtime_unavailable_message(view: View) -> Option<&'static str> {
    match view {
        View::Home | View::Budget | View::Register | View::Reconcile => {
            Some("current data view unavailable: unable to initialize logos runtime")
        }
        View::Rsu => None,
    }
}

#[must_use]
fn home_status_lines(app: &App) -> Vec<String> {
    app.home_snapshot().map_or_else(
        || vec![String::from("Dashboard data unavailable")],
        |snapshot| {
            vec![
                format!("Cashflow: {}", currency(snapshot.cashflow_cents())),
                format!(
                    "Budget Target: {}",
                    snapshot
                        .budget_target_cents()
                        .map_or_else(|| String::from("unconfigured"), currency)
                ),
                format!(
                    "Budget Variance: {}",
                    snapshot
                        .budget_variance_cents()
                        .map_or_else(|| String::from("unconfigured"), currency)
                ),
            ]
        },
    )
}

fn budget_status_lines(app: &App) -> Vec<String> {
    app.budget_snapshot().map_or_else(
        || vec![String::from("Budget data unavailable")],
        |snapshot| {
            vec![
                format!(
                    "Target: {}",
                    snapshot
                        .budget_target_cents()
                        .map_or_else(|| String::from("unconfigured"), currency)
                ),
                format!("Actual: {}", currency(snapshot.actual_expense_cents())),
                format!(
                    "Variance: {}",
                    snapshot
                        .budget_variance_cents()
                        .map_or_else(|| String::from("unconfigured"), currency)
                ),
            ]
        },
    )
}

fn register_status_lines(app: &App) -> Vec<String> {
    app.register_snapshot().map_or_else(
        || vec![String::from("Register data unavailable")],
        |snapshot| {
            vec![
                format!("Balance: {}", currency(snapshot.balance_cents())),
                format!("Recent Rows: {}", snapshot.activity().len()),
            ]
        },
    )
}

fn reconcile_status_lines(app: &App) -> Vec<String> {
    let selected_run = app.selected_reconcile_run();
    vec![
        format!(
            "Runs: {}",
            if selected_run.is_some() {
                String::from("loaded")
            } else {
                String::from("none")
            }
        ),
        format!(
            "Selected Run: {}",
            selected_run.map_or_else(|| String::from("none"), |run| run.run_id().to_owned())
        ),
        format!(
            "Evidence Rows: {}",
            app.selected_reconcile_statement_lines().len()
        ),
    ]
}

#[must_use]
pub fn view_status_lines(app: &App, runtime_available: bool) -> Vec<Line<'static>> {
    let mode = if app.is_scope_editing() {
        "Edit Scope"
    } else {
        "Normal"
    };
    let mut lines = vec![Line::from(format!("Mode: {mode}"))];
    if let Some(message) = app.scope_error_message() {
        lines.push(Line::from(vec![
            Span::styled("Scope Error: ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled(message.to_owned(), Style::default().fg(Color::Red)),
        ]));
    }
    let view_lines = match app.view() {
        View::Home => home_status_lines(app),
        View::Budget => budget_status_lines(app),
        View::Register => register_status_lines(app),
        View::Rsu => vec![String::from("RSU view still placeholder")],
        View::Reconcile => reconcile_status_lines(app),
    };
    lines.extend(view_lines.into_iter().map(Line::from));

    if !runtime_available {
        if let Some(message) = runtime_unavailable_message(app.view()) {
            lines.push(Line::from(Span::styled(
                message.to_owned(),
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
            )));
        }
    }

    lines
}

fn footer_lines(app: &App) -> Vec<Line<'static>> {
    if app.is_scope_editing() {
        return vec![Line::from(
            "[type] edit  [tab/shift-tab or arrows] switch field  [backspace] delete  [enter] apply  [esc] cancel  [ctrl-c] quit",
        )];
    }

    vec![Line::from(
        "[h] home  [b] budget  [r] register  [s] rsu  [c] reconcile  [left/right] tabs  [i] edit scope  [j/k or up/down] move  [q|ctrl-c] quit",
    )]
}
