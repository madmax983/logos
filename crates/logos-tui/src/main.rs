use std::io::{self, Write};

use logos_cli::runtime::CliRuntime;
use logos_tui::{App, View};

fn main() {
    if let Err(err) = run() {
        eprintln!("tui runtime error: {err}");
        std::process::exit(1);
    }
}

fn run() -> io::Result<()> {
    let mut app = App::new();
    let runtime = CliRuntime::new().ok();

    loop {
        if app.view() == View::Reconcile {
            if let Some(source) = runtime.as_ref() {
                app.refresh_reconcile(source);
            }
        }

        println!("{}", app.render_frame());
        if app.view() == View::Reconcile && runtime.is_none() {
            println!("reconcile view unavailable: unable to initialize logos runtime");
        }
        if app.should_exit() {
            break;
        }

        print!("key[h,b,r,s,c,j,k,q] > ");
        io::stdout().flush()?;
        let mut input = String::new();
        if io::stdin().read_line(&mut input)? == 0 {
            app.request_exit();
            continue;
        }

        let Some(key) = input.trim().chars().next() else {
            continue;
        };
        app.handle_key(key);
    }

    Ok(())
}
