use std::io;

use logos_runtime::AppRuntime;
use logos_tui::{App, TerminalSession};

fn main() {
    if let Err(err) = run() {
        eprintln!("tui runtime error: {err}");
        std::process::exit(1);
    }
}

fn run() -> io::Result<()> {
    let mut app = App::new();
    let database_url = std::env::var("DATABASE_URL").unwrap_or_default();
    let runtime = if database_url.is_empty() {
        None
    } else {
        logos_store_pg::PostgresStore::connect(&database_url)
            .ok()
            .map(|store| {
                let state_root = logos_runtime::default_state_root();
                AppRuntime::with_store(
                    store,
                    logos_runtime::default_artifacts_root(&state_root),
                    Some(logos_runtime::default_fetch_config_path(&state_root)),
                )
            })
    };
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
