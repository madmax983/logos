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
        match &self.command {
            Command::Help(topic) => commands::help::show(*topic),
            Command::Aletheia(AletheiaCommand::Start) => commands::aletheia::start(),
            Command::Aletheia(AletheiaCommand::Status) => commands::aletheia::status(),
            Command::Txn(TxnCommand::Add {
                description,
                debit_account,
                credit_account,
                amount_cents,
            }) => commands::txn::add(description, debit_account, credit_account, *amount_cents),
            Command::Analytics(AnalyticsCommand::SnapshotCreate {
                as_of_valid_time_us,
                as_of_tx_time_us,
                schema_version,
                supersedes_artifact_id,
            }) => commands::analytics::snapshot_create(
                *as_of_valid_time_us,
                *as_of_tx_time_us,
                *schema_version,
                supersedes_artifact_id.as_deref(),
            ),
            Command::Analytics(AnalyticsCommand::SnapshotList) => {
                commands::analytics::snapshot_list()
            }
            Command::Analytics(AnalyticsCommand::SnapshotShow { artifact_id }) => {
                commands::analytics::snapshot_show(artifact_id)
            }
            Command::Import(ImportCommand::Pdf {
                file_path,
                account,
                dry_run,
                ocr,
            }) => commands::import::pdf(file_path, account, *dry_run, *ocr),
            Command::Reconcile(ReconcileCommand::Month {
                checking_account,
                month_key,
                opening_balance_cents,
                closing_balance_cents,
            }) => commands::reconcile::month(
                checking_account,
                month_key.as_deref(),
                *opening_balance_cents,
                *closing_balance_cents,
            ),
            Command::Reconcile(ReconcileCommand::List {
                month_key,
                checking_account,
            }) => commands::reconcile::list(month_key.as_deref(), checking_account.as_deref()),
            Command::Reconcile(ReconcileCommand::Show { run_id }) => {
                commands::reconcile::show(run_id)
            }
            Command::Close(CloseCommand::Month {
                month_key,
                checking_account,
                run_id,
                analytics_artifact_id,
            }) => commands::close::month(
                month_key.as_deref(),
                checking_account,
                run_id,
                analytics_artifact_id.as_deref(),
            ),
            Command::Budget(BudgetCommand::Set {
                month_key,
                budget_cents,
                expense_account_prefix,
            }) => {
                commands::budget::set(month_key.as_deref(), *budget_cents, expense_account_prefix)
            }
            Command::Report(ReportCommand::Month {
                checking_account,
                month_key,
            }) => commands::report::month(checking_account, month_key.as_deref()),
        }
    }
}

fn parse_month_key(flag: &str, value: String) -> Result<String, CliError> {
    let bytes = value.as_bytes();
    if bytes.len() != 7 || bytes[4] != b'-' {
        return Err(CliError::InvalidArgValue {
            flag: flag.to_owned(),
            value,
        });
    }

    if !bytes[0..4].iter().all(u8::is_ascii_digit) || !bytes[5..7].iter().all(u8::is_ascii_digit) {
        return Err(CliError::InvalidArgValue {
            flag: flag.to_owned(),
            value,
        });
    }

    let month = value[5..7]
        .parse::<u8>()
        .map_err(|_| CliError::InvalidArgValue {
            flag: flag.to_owned(),
            value: value.clone(),
        })?;
    if !(1..=12).contains(&month) {
        return Err(CliError::InvalidArgValue {
            flag: flag.to_owned(),
            value,
        });
    }

    Ok(value)
}

fn parse_optional_month_flag(args: &[String], flag: &str) -> Result<Option<String>, CliError> {
    parse_optional_flag_value(args, flag)?
        .map(|value| parse_month_key(flag, value))
        .transpose()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Help(HelpTopic),
    Aletheia(AletheiaCommand),
    Txn(TxnCommand),
    Analytics(AnalyticsCommand),
    Import(ImportCommand),
    Reconcile(ReconcileCommand),
    Close(CloseCommand),
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
            Self::Help(HelpTopic::Analytics) => "help.analytics",
            Self::Help(HelpTopic::Import) => "help.import",
            Self::Help(HelpTopic::Reconcile) => "help.reconcile",
            Self::Help(HelpTopic::Close) => "help.close",
            Self::Aletheia(AletheiaCommand::Start) => "aletheia.start",
            Self::Aletheia(AletheiaCommand::Status) => "aletheia.status",
            Self::Txn(TxnCommand::Add { .. }) => "txn.add",
            Self::Analytics(AnalyticsCommand::SnapshotCreate { .. }) => "analytics.snapshot.create",
            Self::Analytics(AnalyticsCommand::SnapshotList) => "analytics.snapshot.list",
            Self::Analytics(AnalyticsCommand::SnapshotShow { .. }) => "analytics.snapshot.show",
            Self::Import(ImportCommand::Pdf { .. }) => "import.pdf",
            Self::Reconcile(ReconcileCommand::Month { .. }) => "reconcile.month",
            Self::Reconcile(ReconcileCommand::List { .. }) => "reconcile.list",
            Self::Reconcile(ReconcileCommand::Show { .. }) => "reconcile.show",
            Self::Close(CloseCommand::Month { .. }) => "close.month",
            Self::Budget(BudgetCommand::Set { .. }) => "budget.set",
            Self::Report(ReportCommand::Month { .. }) => "report.month",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelpTopic {
    General,
    Txn,
    Analytics,
    Budget,
    Report,
    Aletheia,
    Import,
    Reconcile,
    Close,
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
pub enum AnalyticsCommand {
    SnapshotCreate {
        as_of_valid_time_us: Option<i64>,
        as_of_tx_time_us: Option<i64>,
        schema_version: i64,
        supersedes_artifact_id: Option<String>,
    },
    SnapshotList,
    SnapshotShow {
        artifact_id: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportCommand {
    Pdf {
        file_path: String,
        account: String,
        dry_run: bool,
        ocr: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReconcileCommand {
    Month {
        checking_account: String,
        month_key: Option<String>,
        opening_balance_cents: i64,
        closing_balance_cents: i64,
    },
    List {
        month_key: Option<String>,
        checking_account: Option<String>,
    },
    Show {
        run_id: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CloseCommand {
    Month {
        month_key: Option<String>,
        checking_account: String,
        run_id: String,
        analytics_artifact_id: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BudgetCommand {
    Set {
        month_key: Option<String>,
        budget_cents: i64,
        expense_account_prefix: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReportCommand {
    Month {
        checking_account: String,
        month_key: Option<String>,
    },
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
        "analytics" => parse_analytics(&values),
        "import" => parse_import(&values),
        "reconcile" => parse_reconcile(&values),
        "close" => parse_close(&values),
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
            let month_key = parse_optional_month_flag(&args[2..], "--month")?;
            let budget_cents =
                parse_optional_i64_flag(&args[2..], "--budget-cents", DEFAULT_BUDGET_CENTS)?;
            let expense_account_prefix =
                parse_optional_flag_value(&args[2..], "--expense-account-prefix")?
                    .unwrap_or_else(|| DEFAULT_EXPENSE_ACCOUNT_PREFIX.to_owned());
            Ok(ParsedArgs {
                command: Command::Budget(BudgetCommand::Set {
                    month_key,
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

fn parse_analytics(args: &[String]) -> Result<ParsedArgs, CliError> {
    let subcommand = args.get(1).ok_or_else(|| CliError::MissingSubcommand {
        command: "analytics".to_owned(),
    })?;

    match subcommand.as_str() {
        "--help" | "-h" => Ok(ParsedArgs {
            command: Command::Help(HelpTopic::Analytics),
        }),
        "snapshot" => parse_analytics_snapshot(args),
        _ => Err(CliError::UnknownSubcommand {
            command: "analytics".to_owned(),
            subcommand: subcommand.clone(),
        }),
    }
}

fn parse_analytics_snapshot(args: &[String]) -> Result<ParsedArgs, CliError> {
    let action = args.get(2).ok_or_else(|| CliError::MissingSubcommand {
        command: "analytics snapshot".to_owned(),
    })?;

    match action.as_str() {
        "create" => {
            let as_of_valid_time_us = parse_optional_i64_value(&args[3..], "--as-of-valid-us")?;
            let as_of_tx_time_us = parse_optional_i64_value(&args[3..], "--as-of-tx-us")?;
            let schema_version = parse_optional_i64_flag(
                &args[3..],
                "--schema-version",
                crate::runtime::CliRuntime::default_analytics_schema_version(),
            )?;
            let supersedes_artifact_id = parse_optional_flag_value(&args[3..], "--supersedes")?;
            Ok(ParsedArgs {
                command: Command::Analytics(AnalyticsCommand::SnapshotCreate {
                    as_of_valid_time_us,
                    as_of_tx_time_us,
                    schema_version,
                    supersedes_artifact_id,
                }),
            })
        }
        "list" => Ok(ParsedArgs {
            command: Command::Analytics(AnalyticsCommand::SnapshotList),
        }),
        "show" => {
            let artifact_id = parse_flag_value(&args[3..], "--artifact-id")?;
            Ok(ParsedArgs {
                command: Command::Analytics(AnalyticsCommand::SnapshotShow { artifact_id }),
            })
        }
        "--help" | "-h" => Ok(ParsedArgs {
            command: Command::Help(HelpTopic::Analytics),
        }),
        _ => Err(CliError::UnknownSubcommand {
            command: "analytics snapshot".to_owned(),
            subcommand: action.clone(),
        }),
    }
}

fn parse_import(args: &[String]) -> Result<ParsedArgs, CliError> {
    let subcommand = args.get(1).ok_or_else(|| CliError::MissingSubcommand {
        command: "import".to_owned(),
    })?;

    match subcommand.as_str() {
        "--help" | "-h" => Ok(ParsedArgs {
            command: Command::Help(HelpTopic::Import),
        }),
        "pdf" => {
            let file_path = parse_flag_value(&args[2..], "--file")?;
            let account = parse_optional_flag_value(&args[2..], "--account")?
                .unwrap_or_else(|| DEFAULT_CHECKING_ACCOUNT.to_owned());
            let dry_run = parse_flag_present(&args[2..], "--dry-run");
            let ocr = parse_flag_present(&args[2..], "--ocr");
            Ok(ParsedArgs {
                command: Command::Import(ImportCommand::Pdf {
                    file_path,
                    account,
                    dry_run,
                    ocr,
                }),
            })
        }
        _ => Err(CliError::UnknownSubcommand {
            command: "import".to_owned(),
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
            let month_key = parse_optional_month_flag(&args[2..], "--month")?;
            Ok(ParsedArgs {
                command: Command::Report(ReportCommand::Month {
                    checking_account,
                    month_key,
                }),
            })
        }
        _ => Err(CliError::UnknownSubcommand {
            command: "report".to_owned(),
            subcommand: subcommand.clone(),
        }),
    }
}

fn parse_reconcile(args: &[String]) -> Result<ParsedArgs, CliError> {
    let subcommand = args.get(1).ok_or_else(|| CliError::MissingSubcommand {
        command: "reconcile".to_owned(),
    })?;

    match subcommand.as_str() {
        "--help" | "-h" => Ok(ParsedArgs {
            command: Command::Help(HelpTopic::Reconcile),
        }),
        "month" => {
            let checking_account = parse_optional_flag_value(&args[2..], "--checking-account")?
                .unwrap_or_else(|| DEFAULT_CHECKING_ACCOUNT.to_owned());
            let month_key = parse_optional_month_flag(&args[2..], "--month")?;
            let opening_balance_cents =
                parse_required_i64_flag(&args[2..], "--opening-balance-cents")?;
            let closing_balance_cents =
                parse_required_i64_flag(&args[2..], "--closing-balance-cents")?;
            Ok(ParsedArgs {
                command: Command::Reconcile(ReconcileCommand::Month {
                    checking_account,
                    month_key,
                    opening_balance_cents,
                    closing_balance_cents,
                }),
            })
        }
        "list" => {
            let checking_account = parse_optional_flag_value(&args[2..], "--checking-account")?;
            let month_key = parse_optional_month_flag(&args[2..], "--month")?;
            Ok(ParsedArgs {
                command: Command::Reconcile(ReconcileCommand::List {
                    month_key,
                    checking_account,
                }),
            })
        }
        "show" => {
            let run_id = parse_flag_value(&args[2..], "--run-id")?;
            Ok(ParsedArgs {
                command: Command::Reconcile(ReconcileCommand::Show { run_id }),
            })
        }
        _ => Err(CliError::UnknownSubcommand {
            command: "reconcile".to_owned(),
            subcommand: subcommand.clone(),
        }),
    }
}

fn parse_close(args: &[String]) -> Result<ParsedArgs, CliError> {
    let subcommand = args.get(1).ok_or_else(|| CliError::MissingSubcommand {
        command: "close".to_owned(),
    })?;

    match subcommand.as_str() {
        "--help" | "-h" => Ok(ParsedArgs {
            command: Command::Help(HelpTopic::Close),
        }),
        "month" => {
            let month_key = parse_optional_month_flag(&args[2..], "--month")?;
            let checking_account = parse_optional_flag_value(&args[2..], "--checking-account")?
                .unwrap_or_else(|| DEFAULT_CHECKING_ACCOUNT.to_owned());
            let run_id = parse_flag_value(&args[2..], "--run-id")?;
            let analytics_artifact_id =
                parse_optional_flag_value(&args[2..], "--analytics-artifact-id")?;
            Ok(ParsedArgs {
                command: Command::Close(CloseCommand::Month {
                    month_key,
                    checking_account,
                    run_id,
                    analytics_artifact_id,
                }),
            })
        }
        _ => Err(CliError::UnknownSubcommand {
            command: "close".to_owned(),
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
        Some("analytics") => Ok(HelpTopic::Analytics),
        Some("budget") => Ok(HelpTopic::Budget),
        Some("report") => Ok(HelpTopic::Report),
        Some("aletheia") => Ok(HelpTopic::Aletheia),
        Some("import") => Ok(HelpTopic::Import),
        Some("reconcile") => Ok(HelpTopic::Reconcile),
        Some("close") => Ok(HelpTopic::Close),
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

fn parse_flag_present(args: &[String], flag: &str) -> bool {
    args.iter().any(|arg| arg == flag)
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

fn parse_optional_i64_value(args: &[String], flag: &str) -> Result<Option<i64>, CliError> {
    parse_optional_flag_value(args, flag)?
        .map(|value| {
            value.parse::<i64>().map_err(|_| CliError::InvalidArgValue {
                flag: flag.to_owned(),
                value,
            })
        })
        .transpose()
}

fn parse_required_i64_flag(args: &[String], flag: &str) -> Result<i64, CliError> {
    let value = parse_flag_value(args, flag)?;
    value.parse::<i64>().map_err(|_| CliError::InvalidArgValue {
        flag: flag.to_owned(),
        value,
    })
}

fn parse_amount_cents(args: &[String]) -> Result<i64, CliError> {
    let value = parse_flag_value(args, "--amount-cents")?;
    value.parse::<i64>().map_err(|_| CliError::InvalidArgValue {
        flag: "--amount-cents".to_owned(),
        value,
    })
}
