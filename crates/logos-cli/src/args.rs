use core::fmt;

use crate::commands;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CliError {
    MissingCommand,
    MissingSubcommand { command: String },
    UnknownCommand { command: String },
    UnknownSubcommand { command: String, subcommand: String },
    MissingArgValue { flag: String },
    MissingTxnDescription,
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingCommand => write!(f, "missing command"),
            Self::MissingSubcommand { command } => {
                write!(f, "missing subcommand for command '{command}'")
            }
            Self::UnknownCommand { command } => write!(f, "unknown command '{command}'"),
            Self::UnknownSubcommand {
                command,
                subcommand,
            } => write!(f, "unknown subcommand '{subcommand}' for command '{command}'"),
            Self::MissingArgValue { flag } => write!(f, "missing value for argument '{flag}'"),
            Self::MissingTxnDescription => write!(f, "missing transaction description"),
        }
    }
}

impl std::error::Error for CliError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedArgs {
    command: Command,
}

impl ParsedArgs {
    #[must_use]
    pub const fn command(&self) -> &Command {
        &self.command
    }

    #[must_use]
    pub const fn command_path(&self) -> &'static str {
        self.command.path()
    }

    /// Executes the selected command handler.
    ///
    /// # Errors
    ///
    /// Returns parser-level validation errors for incomplete command payloads.
    pub fn execute(&self) -> Result<(), CliError> {
        match self.command() {
            Command::Txn(TxnCommand::Add { description }) => commands::txn::add(description),
            Command::Budget(BudgetCommand::Set) => commands::budget::set(),
            Command::Report(ReportCommand::Month) => commands::report::month(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Txn(TxnCommand),
    Budget(BudgetCommand),
    Report(ReportCommand),
}

impl Command {
    #[must_use]
    pub const fn path(&self) -> &'static str {
        match self {
            Self::Txn(TxnCommand::Add { .. }) => "txn.add",
            Self::Budget(BudgetCommand::Set) => "budget.set",
            Self::Report(ReportCommand::Month) => "report.month",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TxnCommand {
    Add { description: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BudgetCommand {
    Set,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportCommand {
    Month,
}

/// Parses verb-first CLI arguments.
///
/// # Errors
///
/// Returns an error when command/subcommand arguments are invalid or incomplete.
pub fn parse_args<I, S>(argv: I) -> Result<ParsedArgs, CliError>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let mut values: Vec<String> = argv.into_iter().map(Into::into).collect();
    if !values.is_empty() {
        values.remove(0);
    }

    let command = values.first().ok_or(CliError::MissingCommand)?;
    match command.as_str() {
        "txn" => parse_txn(&values),
        "budget" => parse_budget(&values),
        "report" => parse_report(&values),
        _ => Err(CliError::UnknownCommand {
            command: command.clone(),
        }),
    }
}

fn parse_txn(args: &[String]) -> Result<ParsedArgs, CliError> {
    let subcommand = args.get(1).ok_or_else(|| CliError::MissingSubcommand {
        command: "txn".to_owned(),
    })?;

    match subcommand.as_str() {
        "add" => {
            let description = parse_flag_value(&args[2..], "--description")?;
            Ok(ParsedArgs {
                command: Command::Txn(TxnCommand::Add { description }),
            })
        }
        _ => Err(CliError::UnknownSubcommand {
            command: "txn".to_owned(),
            subcommand: subcommand.clone(),
        }),
    }
}

fn parse_budget(args: &[String]) -> Result<ParsedArgs, CliError> {
    let subcommand = args.get(1).ok_or_else(|| CliError::MissingSubcommand {
        command: "budget".to_owned(),
    })?;

    match subcommand.as_str() {
        "set" => Ok(ParsedArgs {
            command: Command::Budget(BudgetCommand::Set),
        }),
        _ => Err(CliError::UnknownSubcommand {
            command: "budget".to_owned(),
            subcommand: subcommand.clone(),
        }),
    }
}

fn parse_report(args: &[String]) -> Result<ParsedArgs, CliError> {
    let subcommand = args.get(1).ok_or_else(|| CliError::MissingSubcommand {
        command: "report".to_owned(),
    })?;

    match subcommand.as_str() {
        "month" => Ok(ParsedArgs {
            command: Command::Report(ReportCommand::Month),
        }),
        _ => Err(CliError::UnknownSubcommand {
            command: "report".to_owned(),
            subcommand: subcommand.clone(),
        }),
    }
}

fn parse_flag_value(args: &[String], flag: &str) -> Result<String, CliError> {
    let idx = args
        .iter()
        .position(|arg| arg == flag)
        .ok_or(CliError::MissingTxnDescription)?;
    let value = args
        .get(idx + 1)
        .ok_or_else(|| CliError::MissingArgValue {
            flag: flag.to_owned(),
        })?;
    Ok(value.clone())
}
