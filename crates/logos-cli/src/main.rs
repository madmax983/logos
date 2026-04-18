use crossterm::style::Stylize;

fn main() {
    if let Err(err) = logos_cli::run(std::env::args()) {
        eprintln!("{}", format!("{err}").red().bold());
        std::process::exit(1);
    }
}
