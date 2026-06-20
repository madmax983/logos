use crossterm::style::Stylize;

fn main() {
    if let Err(err) = logos_cli::run(std::env::args()) {
        eprintln!(
            "\n  {} {}\n",
            " 🛑 ERROR ".white().on_red().bold(),
            err.to_string().red()
        );
        std::process::exit(1);
    }
}
