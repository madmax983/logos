use std::io;

use logos_runtime::AppRuntime;
use logos_tui::{App, terminal::TerminalSession};

fn main() {
    if let Err(err) = run() {
        eprintln!("tui runtime error: {err}");
        std::process::exit(1);
    }
}

fn run() -> io::Result<()> {
    let mut app = App::new();
    let runtime = AppRuntime::new().ok();
    let mut terminal = TerminalSession::enter()?;

    loop {
        if let Some(source) = runtime.as_ref() {
            app.refresh_current_view(source);
        }

        terminal.draw(&app, runtime.is_some())?;
        if app.should_exit() {
            break;
        }

        if let Some(input) = terminal.next_input()? {
            app.handle_input(input);
        }
    }

    Ok(())
}
