//! Logos Command Line Interface
pub(crate) mod args;
pub(crate) mod commands;
pub mod format;
pub(crate) mod runtime;

pub use args::{
    AnalyticsCommand, BudgetCommand, CliError, CloseCommand, Command, FetchCommand, ImportCommand,
    MonthCommand, ParsedArgs, ReconcileCommand, ReportCommand, parse_args,
};

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
