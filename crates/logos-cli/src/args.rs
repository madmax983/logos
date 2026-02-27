use core::fmt;

use crate::commands;

const DEFAULT_BUDGET_CENTS: i64 = 0;
const DEFAULT_EXPENSE_ACCOUNT_PREFIX: &str = "expenses:";
const DEFAULT_CHECKING_ACCOUNT: &str = "assets:checking";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CliError {
    MissingCommand,
    MissingSubcommand { command: String },
    UnknownCommand { command: String },
    UnknownSubcommand { command: String, subcommand: String },
    MissingArgValue { flag: String },
    InvalidArgValue { flag: String, value: String },
    MissingTxnDescription,
    CommandRuntimeFailed { command: String, message: String },
    AletheiaStartFailed { message: String },
    AletheiaStatusFailed { endpoint: String, message: String },
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
            } => write!(
                f,
                "unknown subcommand '{subcommand}' for command '{command}'"
            ),
            Self::MissingArgValue { flag } => write!(f, "missing value for argument '{flag}'"),
            Self::InvalidArgValue { flag, value } => {
                write!(f, "invalid value '{value}' for argument '{flag}'")
            }
            Self::MissingTxnDescription => write!(f, "missing transaction description"),
            Self::CommandRuntimeFailed { command, message } => {
                write!(f, "command '{command}' failed at runtime: {message}")
            }
            Self::AletheiaStartFailed { message } => {
                write!(f, "failed to start aletheia server: {message}")
            }
            Self::AletheiaStatusFailed { endpoint, message } => {
                write!(f, "failed status check at '{endpoint}': {message}")
            }
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
            Command::Help(topic) => commands::help::show(*topic),
            Command::Aletheia(AletheiaCommand::Start) => commands::aletheia::start(),
            Command::Aletheia(AletheiaCommand::Status) => commands::aletheia::status(),
            Command::Txn(TxnCommand::Add {
                description,
                debit_account,
                credit_account,
                amount_cents,
            }) => commands::txn::add(description, debit_account, credit_account, *amount_cents),
            Command::Budget(BudgetCommand::Set {
                budget_cents,
                expense_account_prefix,
            }) => commands::budget::set(*budget_cents, expense_account_prefix),
            Command::Report(ReportCommand::Month { checking_account }) => {
                commands::report::month(checking_account)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Help(HelpTopic),
    Aletheia(AletheiaCommand),
    Txn(TxnCommand),
    Budget(BudgetCommand),
    Report(ReportCommand),
}

impl Command {
    #[must_use]
    pub const fn path(&self) -> &'static str {
        match self {
            Self::Help(HelpTopic::General) => "help",
            Self::Help(HelpTopic::Txn) => "help.txn",
            Self::Help(HelpTopic::Budget) => "help.budget",
            Self::Help(HelpTopic::Report) => "help.report",
            Self::Help(HelpTopic::Aletheia) => "help.aletheia",
            Self::Aletheia(AletheiaCommand::Start) => "aletheia.start",
            Self::Aletheia(AletheiaCommand::Status) => "aletheia.status",
            Self::Txn(TxnCommand::Add { .. }) => "txn.add",
            Self::Budget(BudgetCommand::Set { .. }) => "budget.set",
            Self::Report(ReportCommand::Month { .. }) => "report.month",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelpTopic {
    General,
    Txn,
    Budget,
    Report,
    Aletheia,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AletheiaCommand {
    Start,
    Status,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TxnCommand {
    Add {
        description: String,
        debit_account: String,
        credit_account: String,
        amount_cents: i64,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BudgetCommand {
    Set {
        budget_cents: i64,
        expense_account_prefix: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReportCommand {
    Month { checking_account: String },
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
        "--help" | "-h" | "help" => Ok(ParsedArgs {
            command: Command::Help(parse_help_topic(&values)?),
        }),
        "aletheia" => parse_aletheia(&values),
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
        "--help" | "-h" => Ok(ParsedArgs {
            command: Command::Help(HelpTopic::Txn),
        }),
        "add" => {
            let description = parse_flag_value(&args[2..], "--description")?;
            let debit_account = parse_flag_value(&args[2..], "--debit-account")?;
            let credit_account = parse_flag_value(&args[2..], "--credit-account")?;
            let amount_cents = parse_amount_cents(&args[2..])?;
            Ok(ParsedArgs {
                command: Command::Txn(TxnCommand::Add {
                    description,
                    debit_account,
                    credit_account,
                    amount_cents,
                }),
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
        "--help" | "-h" => Ok(ParsedArgs {
            command: Command::Help(HelpTopic::Budget),
        }),
        "set" => {
            let budget_cents =
                parse_optional_i64_flag(&args[2..], "--budget-cents", DEFAULT_BUDGET_CENTS)?;
            let expense_account_prefix =
                parse_optional_flag_value(&args[2..], "--expense-account-prefix")?
                    .unwrap_or_else(|| DEFAULT_EXPENSE_ACCOUNT_PREFIX.to_owned());
            Ok(ParsedArgs {
                command: Command::Budget(BudgetCommand::Set {
                    budget_cents,
                    expense_account_prefix,
                }),
            })
        }
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
        "--help" | "-h" => Ok(ParsedArgs {
            command: Command::Help(HelpTopic::Report),
        }),
        "month" => {
            let checking_account = parse_optional_flag_value(&args[2..], "--checking-account")?
                .unwrap_or_else(|| DEFAULT_CHECKING_ACCOUNT.to_owned());
            Ok(ParsedArgs {
                command: Command::Report(ReportCommand::Month { checking_account }),
            })
        }
        _ => Err(CliError::UnknownSubcommand {
            command: "report".to_owned(),
            subcommand: subcommand.clone(),
        }),
    }
}

fn parse_aletheia(args: &[String]) -> Result<ParsedArgs, CliError> {
    let subcommand = args.get(1).ok_or_else(|| CliError::MissingSubcommand {
        command: "aletheia".to_owned(),
    })?;

    match subcommand.as_str() {
        "--help" | "-h" => Ok(ParsedArgs {
            command: Command::Help(HelpTopic::Aletheia),
        }),
        "start" => Ok(ParsedArgs {
            command: Command::Aletheia(AletheiaCommand::Start),
        }),
        "status" => Ok(ParsedArgs {
            command: Command::Aletheia(AletheiaCommand::Status),
        }),
        _ => Err(CliError::UnknownSubcommand {
            command: "aletheia".to_owned(),
            subcommand: subcommand.clone(),
        }),
    }
}

fn parse_help_topic(args: &[String]) -> Result<HelpTopic, CliError> {
    match args.get(1).map(String::as_str) {
        None => Ok(HelpTopic::General),
        Some("txn") => Ok(HelpTopic::Txn),
        Some("budget") => Ok(HelpTopic::Budget),
        Some("report") => Ok(HelpTopic::Report),
        Some("aletheia") => Ok(HelpTopic::Aletheia),
        Some(subcommand) => Err(CliError::UnknownSubcommand {
            command: "help".to_owned(),
            subcommand: subcommand.to_owned(),
        }),
    }
}

fn parse_flag_value(args: &[String], flag: &str) -> Result<String, CliError> {
    let idx = args
        .iter()
        .position(|arg| arg == flag)
        .ok_or_else(|| CliError::MissingArgValue {
            flag: flag.to_owned(),
        })?;
    let value = args.get(idx + 1).ok_or_else(|| CliError::MissingArgValue {
        flag: flag.to_owned(),
    })?;
    Ok(value.clone())
}

fn parse_optional_flag_value(args: &[String], flag: &str) -> Result<Option<String>, CliError> {
    let Some(idx) = args.iter().position(|arg| arg == flag) else {
        return Ok(None);
    };

    let value = args.get(idx + 1).ok_or_else(|| CliError::MissingArgValue {
        flag: flag.to_owned(),
    })?;
    Ok(Some(value.clone()))
}

fn parse_optional_i64_flag(
    args: &[String],
    flag: &str,
    default_value: i64,
) -> Result<i64, CliError> {
    parse_optional_flag_value(args, flag)?.map_or(Ok(default_value), |value| {
        value.parse::<i64>().map_err(|_| CliError::InvalidArgValue {
            flag: flag.to_owned(),
            value,
        })
    })
}

fn parse_amount_cents(args: &[String]) -> Result<i64, CliError> {
    let value = parse_flag_value(args, "--amount-cents")?;
    value.parse::<i64>().map_err(|_| CliError::InvalidArgValue {
        flag: "--amount-cents".to_owned(),
        value,
    })
}
