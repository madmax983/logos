fn main() {
    if let Err(err) = logos_cli::run(std::env::args()) {
        eprintln!("{err}");
        std::process::exit(1);
    }
}
