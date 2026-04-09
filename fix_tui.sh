sed -i 's/pub mod app;/pub(crate) mod app;/g' crates/logos-tui/src/lib.rs
sed -i 's/pub mod ui;/pub(crate) mod ui;/g' crates/logos-tui/src/lib.rs
# terminal has to be public because of logos_tui::terminal::TerminalSession
# but wait! TerminalSession could be exported directly
sed -i 's/pub use terminal::{/pub use terminal::{TerminalSession, /g' crates/logos-tui/src/lib.rs
sed -i 's/use logos_tui::{App, terminal::TerminalSession};/use logos_tui::{App, TerminalSession};/g' crates/logos-tui/src/main.rs
sed -i 's/pub mod terminal;/pub(crate) mod terminal;/g' crates/logos-tui/src/lib.rs
