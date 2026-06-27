#[cfg(not(test))]
use crossterm::style::Stylize;

#[cfg(not(test))]
fn main() {
    if let Err(err) = logos_cli::run(std::env::args()) {
        eprintln!("{}", format!("Error: {err}").red().bold());
        std::process::exit(1);
    }
}
