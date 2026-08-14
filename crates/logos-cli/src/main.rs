use crossterm::style::Stylize;

fn main() {
    if let Err(err) = logos_cli::run(std::env::args()) {
        eprintln!("{} {}", "✘".red().bold(), format!("{err}").red());
        std::process::exit(1);
    }
}
