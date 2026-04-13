#![allow(missing_docs)]
pub mod args;
pub mod commands;

pub use args::{CliError, ParsedArgs, parse_args};

/// Runs the CLI using the provided process arguments.
///
/// # Errors
///
/// Returns an error when parsing fails or command dispatch is invalid.
pub fn run<I, S>(argv: I) -> Result<(), CliError>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let parsed = parse_args(argv)?;
    parsed.execute()
}
