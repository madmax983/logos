//! Command Line Argument Parsing
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
    MissingRequiredArg { flag: String },
    MissingArgValue { flag: String },
    InvalidArgValue { flag: String, value: String },
    MissingTxnDescription,
    CommandRuntimeFailed { command: String, message: String },
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingCommand => write!(f, "Missing command."),
            Self::MissingSubcommand { command } => {
                write!(f, "Missing subcommand for command '{command}'.")
            }
            Self::UnknownCommand { command } => write!(f, "Unknown command '{command}'."),
            Self::UnknownSubcommand {
                command,
                subcommand,
            } => write!(
                f,
                "Unknown subcommand '{subcommand}' for command '{command}'."
            ),
            Self::MissingRequiredArg { flag } => write!(f, "Missing required argument '{flag}'."),
            Self::MissingArgValue { flag } => write!(f, "Missing value for argument '{flag}'."),
            Self::InvalidArgValue { flag, value } => {
                write!(f, "Invalid value '{value}' for argument '{flag}'.")
            }
            Self::MissingTxnDescription => write!(f, "Missing transaction description."),
            Self::CommandRuntimeFailed {
                command: _,
                message,
            } => {
                write!(f, "{message}")
            }
        }
    }
}

impl std::error::Error for CliError {}

#[derive(Debug, Clone, PartialEq)]
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
        execute_command(&self.command)
    }
}

fn execute_command(command: &Command) -> Result<(), CliError> {
    match command {
        Command::Plan(command) => execute_plan_command(command),
        Command::Help(topic) => commands::help::show(*topic),
        Command::Db(command) => execute_db_command(*command),
        Command::Txn(command) => execute_txn_command(command),
        Command::Analytics(command) => execute_analytics_command(command),
        Command::Import(command) => execute_import_command(command),
        Command::Fetch(command) => execute_fetch_command(command),
        Command::Reconcile(command) => execute_reconcile_command(command),
        Command::Month(command) => execute_month_command(command),
        Command::Close(command) => execute_close_command(command),
        Command::Budget(command) => execute_budget_command(command),
        Command::Report(command) => execute_report_command(command),
    }
}

fn execute_db_command(command: DbCommand) -> Result<(), CliError> {
    match command {
        DbCommand::Migrate => commands::db::migrate(),
        DbCommand::Status => commands::db::status(),
    }
}

fn execute_txn_command(command: &TxnCommand) -> Result<(), CliError> {
    match command {
        TxnCommand::Add {
            description,
            debit_account,
            credit_account,
            amount_cents,
        } => commands::txn::add(description, debit_account, credit_account, *amount_cents),
        TxnCommand::Correct {
            supersedes_id,
            reason,
        } => commands::txn::correct(supersedes_id, reason),
    }
}

fn execute_analytics_command(command: &AnalyticsCommand) -> Result<(), CliError> {
    match command {
        AnalyticsCommand::SnapshotCreate {
            as_of_valid_time_us,
            as_of_tx_time_us,
            schema_version,
            supersedes_artifact_id,
        } => commands::analytics::snapshot_create(
            *as_of_valid_time_us,
            *as_of_tx_time_us,
            *schema_version,
            supersedes_artifact_id.as_deref(),
        ),
        AnalyticsCommand::SnapshotList => commands::analytics::snapshot_list(),
        AnalyticsCommand::SnapshotShow { artifact_id } => {
            commands::analytics::snapshot_show(artifact_id)
        }
        AnalyticsCommand::Sankey => commands::analytics::sankey(),
        AnalyticsCommand::FireSim {
            monthly_expenses_cents,
            liquid_assets_cents,
            monthly_savings_cents,
        } => commands::analytics::fire_sim(
            *monthly_expenses_cents,
            *liquid_assets_cents,
            *monthly_savings_cents,
        ),
        AnalyticsCommand::NetWorthProject {
            initial_net_worth_cents,
            monthly_savings_cents,
            months,
        } => commands::analytics::net_worth_project(
            *initial_net_worth_cents,
            *monthly_savings_cents,
            *months,
        ),
    }
}

fn execute_import_command(command: &ImportCommand) -> Result<(), CliError> {
    match command {
        ImportCommand::Pdf {
            file_path,
            account,
            dry_run,
            ocr,
        } => commands::import::pdf(file_path, account, *dry_run, *ocr),
        ImportCommand::Csv {
            file_path,
            source_id,
            timestamp_idx,
            amount_idx,
            memo_idx,
            account_idx,
            category_idx,
            skip_header,
            dry_run,
        } => commands::import::csv(
            file_path,
            source_id.as_deref(),
            *timestamp_idx,
            *amount_idx,
            *memo_idx,
            *account_idx,
            *category_idx,
            *skip_header,
            *dry_run,
        ),
    }
}

fn execute_fetch_command(command: &FetchCommand) -> Result<(), CliError> {
    match command {
        FetchCommand::ListRuns {
            month_key,
            checking_account,
        } => commands::fetch::list_runs(month_key.as_deref(), checking_account.as_deref()),
        FetchCommand::ShowRun { run_id } => commands::fetch::show_run(run_id),
    }
}

fn execute_reconcile_command(command: &ReconcileCommand) -> Result<(), CliError> {
    match command {
        ReconcileCommand::Month {
            checking_account,
            month_key,
            opening_balance_cents,
            closing_balance_cents,
        } => commands::reconcile::month(
            checking_account,
            month_key.as_deref(),
            *opening_balance_cents,
            *closing_balance_cents,
        ),
        ReconcileCommand::List {
            month_key,
            checking_account,
        } => commands::reconcile::list(month_key.as_deref(), checking_account.as_deref()),
        ReconcileCommand::Show { run_id } => commands::reconcile::show(run_id),
    }
}

fn execute_month_command(command: &MonthCommand) -> Result<(), CliError> {
    match command {
        MonthCommand::Autopilot {
            month_key,
            checking_account,
            opening_balance_cents,
            closing_balance_cents,
            statement_pdf,
            ocr,
            allow_variance,
            analytics_artifact_id,
            confirm_close,
        } => commands::month::autopilot(
            month_key.as_deref(),
            checking_account,
            *opening_balance_cents,
            *closing_balance_cents,
            statement_pdf.as_deref(),
            *ocr,
            *allow_variance,
            analytics_artifact_id.as_deref(),
            *confirm_close,
        ),
    }
}

fn execute_close_command(command: &CloseCommand) -> Result<(), CliError> {
    match command {
        CloseCommand::Month {
            month_key,
            checking_account,
            run_id,
            analytics_artifact_id,
        } => commands::close::month(
            month_key.as_deref(),
            checking_account,
            run_id,
            analytics_artifact_id.as_deref(),
        ),
    }
}

fn execute_budget_command(command: &BudgetCommand) -> Result<(), CliError> {
    match command {
        BudgetCommand::Set {
            month_key,
            budget_cents,
            expense_account_prefix,
        } => commands::budget::set(month_key.as_deref(), *budget_cents, expense_account_prefix),
        BudgetCommand::RsuPlan {
            month_key,
            quarterly_units,
            days_to_vest,
            bear_price_cents,
            base_price_cents,
            bull_price_cents,
            fixed_commitments_cents,
            reserve_sweep_pct,
            investing_sweep_pct,
        } => commands::budget::rsu_plan(
            month_key.as_deref(),
            *quarterly_units,
            *days_to_vest,
            *bear_price_cents,
            *base_price_cents,
            *bull_price_cents,
            *fixed_commitments_cents,
            *reserve_sweep_pct,
            *investing_sweep_pct,
        ),
        BudgetCommand::MonteCarlo {
            initial_cents,
            monthly_contribution_cents,
            annual_mean_return,
            annual_volatility,
            seed,
            months,
            paths,
        } => commands::budget::monte_carlo(
            *initial_cents,
            *monthly_contribution_cents,
            *annual_mean_return,
            *annual_volatility,
            *seed,
            *months,
            *paths,
        ),
    }
}

fn execute_report_command(command: &ReportCommand) -> Result<(), CliError> {
    match command {
        ReportCommand::Month {
            checking_account,
            month_key,
        } => commands::report::month(checking_account, month_key.as_deref()),
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

    let Ok(month) = value[5..7].parse::<u8>() else {
        return Err(CliError::InvalidArgValue {
            flag: flag.to_owned(),
            value: value.clone(),
        });
    };
    if !(1..=12).contains(&month) {
        return Err(CliError::InvalidArgValue {
            flag: flag.to_owned(),
            value,
        });
    }

    Ok(value)
}

fn parse_optional_month_flag(args: &[String], flag: &str) -> Result<Option<String>, CliError> {
    let Some(value) = parse_optional_flag_value(args, flag)? else {
        return Ok(None);
    };

    Ok(Some(parse_month_key(flag, value)?))
}

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Help(HelpTopic),
    Plan(PlanCommand),
    Db(DbCommand),
    Txn(TxnCommand),
    Analytics(AnalyticsCommand),
    Import(ImportCommand),
    Fetch(FetchCommand),
    Reconcile(ReconcileCommand),
    Month(MonthCommand),
    Close(CloseCommand),
    Budget(BudgetCommand),
    Report(ReportCommand),
}

impl Command {
    #[must_use]
    pub const fn path(&self) -> &'static str {
        match self {
            Self::Help(HelpTopic::General) => "help",
            Self::Plan(PlanCommand::Fire { .. }) => "plan.fire",
            Self::Plan(PlanCommand::Project { .. }) => "plan.project",
            Self::Help(HelpTopic::Txn) => "help.txn",
            Self::Help(HelpTopic::Budget) => "help.budget",
            Self::Help(HelpTopic::Report) => "help.report",
            Self::Help(HelpTopic::Db) => "help.db",
            Self::Help(HelpTopic::Analytics) => "help.analytics",
            Self::Help(HelpTopic::Import) => "help.import",
            Self::Help(HelpTopic::Fetch) => "help.fetch",
            Self::Help(HelpTopic::Reconcile) => "help.reconcile",
            Self::Help(HelpTopic::Month) => "help.month",
            Self::Help(HelpTopic::Close) => "help.close",
            Self::Help(HelpTopic::Plan) => "help.plan",
            Self::Db(DbCommand::Migrate) => "db.migrate",
            Self::Db(DbCommand::Status) => "db.status",
            Self::Txn(TxnCommand::Add { .. }) => "txn.add",
            Self::Txn(TxnCommand::Correct { .. }) => "txn.correct",
            Self::Analytics(AnalyticsCommand::SnapshotCreate { .. }) => "analytics.snapshot.create",
            Self::Analytics(AnalyticsCommand::SnapshotList) => "analytics.snapshot.list",
            Self::Analytics(AnalyticsCommand::SnapshotShow { .. }) => "analytics.snapshot.show",
            Self::Analytics(AnalyticsCommand::Sankey) => "analytics.sankey",
            Self::Analytics(AnalyticsCommand::FireSim { .. }) => "analytics.fire-sim",
            Self::Analytics(AnalyticsCommand::NetWorthProject { .. }) => "analytics.net-worth",
            Self::Import(ImportCommand::Pdf { .. }) => "import.pdf",
            Self::Import(ImportCommand::Csv { .. }) => "import.csv",
            Self::Fetch(FetchCommand::ListRuns { .. }) => "fetch.list",
            Self::Fetch(FetchCommand::ShowRun { .. }) => "fetch.show",
            Self::Reconcile(ReconcileCommand::Month { .. }) => "reconcile.month",
            Self::Reconcile(ReconcileCommand::List { .. }) => "reconcile.list",
            Self::Reconcile(ReconcileCommand::Show { .. }) => "reconcile.show",
            Self::Month(MonthCommand::Autopilot { .. }) => "month.autopilot",
            Self::Close(CloseCommand::Month { .. }) => "close.month",
            Self::Budget(BudgetCommand::Set { .. }) => "budget.set",
            Self::Budget(BudgetCommand::RsuPlan { .. }) => "budget.rsu-plan",
            Self::Budget(BudgetCommand::MonteCarlo { .. }) => "budget.monte-carlo",
            Self::Report(ReportCommand::Month { .. }) => "report.month",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelpTopic {
    General,
    Plan,
    Txn,
    Analytics,
    Budget,
    Report,
    Db,
    Import,
    Fetch,
    Reconcile,
    Month,
    Close,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DbCommand {
    Migrate,
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
    Correct {
        supersedes_id: String,
        reason: String,
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
    Sankey,
    FireSim {
        monthly_expenses_cents: i64,
        liquid_assets_cents: i64,
        monthly_savings_cents: i64,
    },
    NetWorthProject {
        initial_net_worth_cents: i64,
        monthly_savings_cents: i64,
        months: u16,
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
    Csv {
        file_path: String,
        source_id: Option<String>,
        timestamp_idx: usize,
        amount_idx: usize,
        memo_idx: usize,
        account_idx: usize,
        category_idx: usize,
        skip_header: bool,
        dry_run: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FetchCommand {
    ListRuns {
        month_key: Option<String>,
        checking_account: Option<String>,
    },
    ShowRun {
        run_id: String,
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
pub enum MonthCommand {
    Autopilot {
        month_key: Option<String>,
        checking_account: String,
        opening_balance_cents: Option<i64>,
        closing_balance_cents: Option<i64>,
        statement_pdf: Option<String>,
        ocr: bool,
        allow_variance: bool,
        analytics_artifact_id: Option<String>,
        confirm_close: bool,
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

#[derive(Debug, Clone, PartialEq)]
pub enum BudgetCommand {
    Set {
        month_key: Option<String>,
        budget_cents: i64,
        expense_account_prefix: String,
    },
    RsuPlan {
        month_key: Option<String>,
        quarterly_units: u32,
        days_to_vest: u16,
        bear_price_cents: i64,
        base_price_cents: i64,
        bull_price_cents: i64,
        fixed_commitments_cents: i64,
        reserve_sweep_pct: u8,
        investing_sweep_pct: u8,
    },
    MonteCarlo {
        initial_cents: i64,
        monthly_contribution_cents: i64,
        annual_mean_return: f64,
        annual_volatility: f64,
        seed: u64,
        months: u16,
        paths: u32,
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
        "db" => parse_db(&values),
        "plan" => parse_plan(&values),
        "txn" => parse_txn(&values),
        "analytics" => parse_analytics(&values),
        "import" => parse_import(&values),
        "fetch" => parse_fetch(&values),
        "reconcile" => parse_reconcile(&values),
        "month" => parse_month(&values),
        "close" => parse_close(&values),
        "budget" => parse_budget(&values),
        "report" => parse_report(&values),
        _ => Err(CliError::UnknownCommand {
            command: command.clone(),
        }),
    }
}

fn parse_txn(args: &[String]) -> Result<ParsedArgs, CliError> {
    if parse_flag_present(args, "--help") || parse_flag_present(args, "-h") {
        return Ok(ParsedArgs {
            command: Command::Help(HelpTopic::Txn),
        });
    }

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
        "correct" => {
            let supersedes_id = parse_flag_value(&args[2..], "--supersedes-id")?;
            let reason = parse_flag_value(&args[2..], "--reason")?;
            Ok(ParsedArgs {
                command: Command::Txn(TxnCommand::Correct {
                    supersedes_id,
                    reason,
                }),
            })
        }
        _ => Err(CliError::UnknownSubcommand {
            command: "txn".to_owned(),
            subcommand: subcommand.clone(),
        }),
    }
}

fn parse_budget_set(args: &[String]) -> Result<ParsedArgs, CliError> {
    let month_key = parse_optional_month_flag(&args[2..], "--month")?;
    let budget_cents =
        parse_optional_parsed_flag::<i64>(&args[2..], "--budget-cents", DEFAULT_BUDGET_CENTS)?;
    let expense_account_prefix = parse_optional_flag_value_with_default(
        &args[2..],
        "--expense-account-prefix",
        DEFAULT_EXPENSE_ACCOUNT_PREFIX,
    )?;
    Ok(ParsedArgs {
        command: Command::Budget(BudgetCommand::Set {
            month_key,
            budget_cents,
            expense_account_prefix,
        }),
    })
}

fn parse_budget_rsu_plan(args: &[String]) -> Result<ParsedArgs, CliError> {
    let month_key = parse_optional_month_flag(&args[2..], "--month")?;
    let quarterly_units = parse_required_parsed_flag::<u32>(&args[2..], "--quarterly-units")?;
    let days_to_vest = parse_optional_parsed_flag::<u16>(&args[2..], "--days-to-vest", 45)?;
    let bear_price_cents = parse_required_parsed_flag::<i64>(&args[2..], "--bear-price-cents")?;
    let base_price_cents = parse_required_parsed_flag::<i64>(&args[2..], "--base-price-cents")?;
    let bull_price_cents = parse_required_parsed_flag::<i64>(&args[2..], "--bull-price-cents")?;
    let fixed_commitments_cents =
        parse_optional_parsed_flag::<i64>(&args[2..], "--fixed-commitments-cents", 0)?;
    let reserve_sweep_pct =
        parse_optional_parsed_flag::<u8>(&args[2..], "--reserve-sweep-pct", 60)?;
    let investing_sweep_pct =
        parse_optional_parsed_flag::<u8>(&args[2..], "--investing-sweep-pct", 30)?;
    Ok(ParsedArgs {
        command: Command::Budget(BudgetCommand::RsuPlan {
            month_key,
            quarterly_units,
            days_to_vest,
            bear_price_cents,
            base_price_cents,
            bull_price_cents,
            fixed_commitments_cents,
            reserve_sweep_pct,
            investing_sweep_pct,
        }),
    })
}

fn parse_budget_monte_carlo(args: &[String]) -> Result<ParsedArgs, CliError> {
    let initial_cents = parse_required_parsed_flag::<i64>(&args[2..], "--initial-cents")?;
    let monthly_contribution_cents =
        parse_required_parsed_flag::<i64>(&args[2..], "--monthly-contribution-cents")?;
    let annual_mean_return = parse_required_parsed_flag::<f64>(&args[2..], "--annual-mean-return")?;
    let annual_volatility = parse_required_parsed_flag::<f64>(&args[2..], "--annual-volatility")?;
    let seed = parse_optional_parsed_flag::<u64>(&args[2..], "--seed", 42)?;
    let months = parse_required_parsed_flag::<u16>(&args[2..], "--months")?;
    let paths = parse_required_parsed_flag::<u32>(&args[2..], "--paths")?;

    Ok(ParsedArgs {
        command: Command::Budget(BudgetCommand::MonteCarlo {
            initial_cents,
            monthly_contribution_cents,
            annual_mean_return,
            annual_volatility,
            seed,
            months,
            paths,
        }),
    })
}

fn parse_budget(args: &[String]) -> Result<ParsedArgs, CliError> {
    if parse_flag_present(args, "--help") || parse_flag_present(args, "-h") {
        return Ok(ParsedArgs {
            command: Command::Help(HelpTopic::Budget),
        });
    }

    let subcommand = args.get(1).ok_or_else(|| CliError::MissingSubcommand {
        command: "budget".to_owned(),
    })?;

    match subcommand.as_str() {
        "--help" | "-h" => Ok(ParsedArgs {
            command: Command::Help(HelpTopic::Budget),
        }),
        "set" => parse_budget_set(args),
        "rsu-plan" => parse_budget_rsu_plan(args),
        "monte-carlo" => parse_budget_monte_carlo(args),
        _ => Err(CliError::UnknownSubcommand {
            command: "budget".to_owned(),
            subcommand: subcommand.clone(),
        }),
    }
}

fn parse_analytics(args: &[String]) -> Result<ParsedArgs, CliError> {
    if parse_flag_present(args, "--help") || parse_flag_present(args, "-h") {
        return Ok(ParsedArgs {
            command: Command::Help(HelpTopic::Analytics),
        });
    }

    let subcommand = args.get(1).ok_or_else(|| CliError::MissingSubcommand {
        command: "analytics".to_owned(),
    })?;

    match subcommand.as_str() {
        "--help" | "-h" => Ok(ParsedArgs {
            command: Command::Help(HelpTopic::Analytics),
        }),
        "snapshot" => parse_analytics_snapshot(args),
        "sankey" => Ok(ParsedArgs {
            command: Command::Analytics(AnalyticsCommand::Sankey),
        }),
        "fire-sim" => {
            let monthly_expenses_cents =
                parse_required_parsed_flag(&args[2..], "--monthly-expenses-cents")?;
            let liquid_assets_cents =
                parse_required_parsed_flag(&args[2..], "--liquid-assets-cents")?;
            let monthly_savings_cents =
                parse_required_parsed_flag(&args[2..], "--monthly-savings-cents")?;
            Ok(ParsedArgs {
                command: Command::Analytics(AnalyticsCommand::FireSim {
                    monthly_expenses_cents,
                    liquid_assets_cents,
                    monthly_savings_cents,
                }),
            })
        }
        "net-worth" => {
            let initial_net_worth_cents =
                parse_required_parsed_flag(&args[2..], "--initial-net-worth-cents")?;
            let monthly_savings_cents =
                parse_required_parsed_flag(&args[2..], "--monthly-savings-cents")?;
            let months = parse_required_parsed_flag(&args[2..], "--months")?;
            Ok(ParsedArgs {
                command: Command::Analytics(AnalyticsCommand::NetWorthProject {
                    initial_net_worth_cents,
                    monthly_savings_cents,
                    months,
                }),
            })
        }
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
            let as_of_valid_time_us =
                parse_optional_parsed_value::<i64>(&args[3..], "--as-of-valid-us")?;
            let as_of_tx_time_us = parse_optional_parsed_value::<i64>(&args[3..], "--as-of-tx-us")?;
            let schema_version = parse_optional_parsed_flag::<i64>(
                &args[3..],
                "--schema-version",
                logos_runtime::AppRuntime::<logos_store_pg::PostgresStore>::default_analytics_schema_version(),
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

fn parse_import_pdf(args: &[String]) -> Result<ParsedArgs, CliError> {
    let file_path = parse_flag_value(&args[2..], "--file")?;
    let account =
        parse_optional_flag_value_with_default(&args[2..], "--account", DEFAULT_CHECKING_ACCOUNT)?;
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

fn parse_import_csv(args: &[String]) -> Result<ParsedArgs, CliError> {
    let file_path = parse_flag_value(&args[2..], "--file")?;
    let source_id = parse_optional_flag_value(&args[2..], "--source-id")?;
    let timestamp_idx = parse_optional_parsed_flag::<usize>(&args[2..], "--timestamp-idx", 0)?;
    let amount_idx = parse_optional_parsed_flag::<usize>(&args[2..], "--amount-idx", 1)?;
    let memo_idx = parse_optional_parsed_flag::<usize>(&args[2..], "--memo-idx", 2)?;
    let account_idx = parse_optional_parsed_flag::<usize>(&args[2..], "--account-idx", 3)?;
    let category_idx = parse_optional_parsed_flag::<usize>(&args[2..], "--category-idx", 4)?;
    let skip_header = parse_flag_present(&args[2..], "--skip-header");
    let dry_run = parse_flag_present(&args[2..], "--dry-run");
    Ok(ParsedArgs {
        command: Command::Import(ImportCommand::Csv {
            file_path,
            source_id,
            timestamp_idx,
            amount_idx,
            memo_idx,
            account_idx,
            category_idx,
            skip_header,
            dry_run,
        }),
    })
}

fn parse_import(args: &[String]) -> Result<ParsedArgs, CliError> {
    if parse_flag_present(args, "--help") || parse_flag_present(args, "-h") {
        return Ok(ParsedArgs {
            command: Command::Help(HelpTopic::Import),
        });
    }

    let subcommand = args.get(1).ok_or_else(|| CliError::MissingSubcommand {
        command: "import".to_owned(),
    })?;

    match subcommand.as_str() {
        "--help" | "-h" => Ok(ParsedArgs {
            command: Command::Help(HelpTopic::Import),
        }),
        "pdf" => parse_import_pdf(args),
        "csv" => parse_import_csv(args),
        _ => Err(CliError::UnknownSubcommand {
            command: "import".to_owned(),
            subcommand: subcommand.clone(),
        }),
    }
}

fn parse_fetch(args: &[String]) -> Result<ParsedArgs, CliError> {
    let subcommand = args.get(1).ok_or_else(|| CliError::MissingSubcommand {
        command: "fetch".to_owned(),
    })?;

    match subcommand.as_str() {
        "--help" | "-h" => Ok(ParsedArgs {
            command: Command::Help(HelpTopic::Fetch),
        }),
        "list-runs" => {
            let checking_account = parse_optional_flag_value(&args[2..], "--checking-account")?;
            let month_key = parse_optional_month_flag(&args[2..], "--month")?;
            Ok(ParsedArgs {
                command: Command::Fetch(FetchCommand::ListRuns {
                    month_key,
                    checking_account,
                }),
            })
        }
        "show-run" => {
            let run_id = parse_flag_value(&args[2..], "--run-id")?;
            Ok(ParsedArgs {
                command: Command::Fetch(FetchCommand::ShowRun { run_id }),
            })
        }
        _ => Err(CliError::UnknownSubcommand {
            command: "fetch".to_owned(),
            subcommand: subcommand.clone(),
        }),
    }
}

fn parse_report(args: &[String]) -> Result<ParsedArgs, CliError> {
    if parse_flag_present(args, "--help") || parse_flag_present(args, "-h") {
        return Ok(ParsedArgs {
            command: Command::Help(HelpTopic::Report),
        });
    }

    let subcommand = args.get(1).ok_or_else(|| CliError::MissingSubcommand {
        command: "report".to_owned(),
    })?;

    match subcommand.as_str() {
        "--help" | "-h" => Ok(ParsedArgs {
            command: Command::Help(HelpTopic::Report),
        }),
        "month" => {
            let checking_account = parse_optional_flag_value_with_default(
                &args[2..],
                "--checking-account",
                DEFAULT_CHECKING_ACCOUNT,
            )?;
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

fn parse_reconcile_month(args: &[String]) -> Result<ParsedArgs, CliError> {
    let checking_account = parse_optional_flag_value_with_default(
        &args[2..],
        "--checking-account",
        DEFAULT_CHECKING_ACCOUNT,
    )?;
    let month_key = parse_optional_month_flag(&args[2..], "--month")?;
    let opening_balance_cents =
        parse_required_parsed_flag::<i64>(&args[2..], "--opening-balance-cents")?;
    let closing_balance_cents =
        parse_required_parsed_flag::<i64>(&args[2..], "--closing-balance-cents")?;
    Ok(ParsedArgs {
        command: Command::Reconcile(ReconcileCommand::Month {
            checking_account,
            month_key,
            opening_balance_cents,
            closing_balance_cents,
        }),
    })
}

fn parse_reconcile_list(args: &[String]) -> Result<ParsedArgs, CliError> {
    let checking_account = parse_optional_flag_value(&args[2..], "--checking-account")?;
    let month_key = parse_optional_month_flag(&args[2..], "--month")?;
    Ok(ParsedArgs {
        command: Command::Reconcile(ReconcileCommand::List {
            month_key,
            checking_account,
        }),
    })
}

fn parse_reconcile_show(args: &[String]) -> Result<ParsedArgs, CliError> {
    let run_id = parse_flag_value(&args[2..], "--run-id")?;
    Ok(ParsedArgs {
        command: Command::Reconcile(ReconcileCommand::Show { run_id }),
    })
}

fn parse_reconcile(args: &[String]) -> Result<ParsedArgs, CliError> {
    if parse_flag_present(args, "--help") || parse_flag_present(args, "-h") {
        return Ok(ParsedArgs {
            command: Command::Help(HelpTopic::Reconcile),
        });
    }

    let subcommand = args.get(1).ok_or_else(|| CliError::MissingSubcommand {
        command: "reconcile".to_owned(),
    })?;

    match subcommand.as_str() {
        "--help" | "-h" => Ok(ParsedArgs {
            command: Command::Help(HelpTopic::Reconcile),
        }),
        "month" => parse_reconcile_month(args),
        "list" => parse_reconcile_list(args),
        "show" => parse_reconcile_show(args),
        _ => Err(CliError::UnknownSubcommand {
            command: "reconcile".to_owned(),
            subcommand: subcommand.clone(),
        }),
    }
}

fn parse_month_autopilot(args: &[String]) -> Result<ParsedArgs, CliError> {
    let month_key = parse_optional_month_flag(&args[2..], "--month")?;
    let checking_account = parse_optional_flag_value_with_default(
        &args[2..],
        "--checking-account",
        DEFAULT_CHECKING_ACCOUNT,
    )?;
    let (opening_balance_cents, closing_balance_cents) = parse_paired_i64_flags(
        &args[2..],
        "--opening-balance-cents",
        "--closing-balance-cents",
    )?;
    let statement_pdf = parse_optional_flag_value(&args[2..], "--statement-pdf")?;
    let ocr = parse_flag_present(&args[2..], "--ocr");
    let allow_variance = parse_flag_present(&args[2..], "--allow-variance");
    let analytics_artifact_id = parse_optional_flag_value(&args[2..], "--analytics-artifact-id")?;
    let confirm_close = parse_flag_present(&args[2..], "--confirm-close");
    Ok(ParsedArgs {
        command: Command::Month(MonthCommand::Autopilot {
            month_key,
            checking_account,
            opening_balance_cents,
            closing_balance_cents,
            statement_pdf,
            ocr,
            allow_variance,
            analytics_artifact_id,
            confirm_close,
        }),
    })
}

fn parse_month(args: &[String]) -> Result<ParsedArgs, CliError> {
    if parse_flag_present(args, "--help") || parse_flag_present(args, "-h") {
        return Ok(ParsedArgs {
            command: Command::Help(HelpTopic::Month),
        });
    }

    let subcommand = args.get(1).ok_or_else(|| CliError::MissingSubcommand {
        command: "month".to_owned(),
    })?;

    match subcommand.as_str() {
        "--help" | "-h" => Ok(ParsedArgs {
            command: Command::Help(HelpTopic::Month),
        }),
        "autopilot" => parse_month_autopilot(args),
        _ => Err(CliError::UnknownSubcommand {
            command: "month".to_owned(),
            subcommand: subcommand.clone(),
        }),
    }
}

fn parse_close(args: &[String]) -> Result<ParsedArgs, CliError> {
    if parse_flag_present(args, "--help") || parse_flag_present(args, "-h") {
        return Ok(ParsedArgs {
            command: Command::Help(HelpTopic::Close),
        });
    }

    let subcommand = args.get(1).ok_or_else(|| CliError::MissingSubcommand {
        command: "close".to_owned(),
    })?;

    match subcommand.as_str() {
        "--help" | "-h" => Ok(ParsedArgs {
            command: Command::Help(HelpTopic::Close),
        }),
        "month" => {
            let month_key = parse_optional_month_flag(&args[2..], "--month")?;
            let checking_account = parse_optional_flag_value_with_default(
                &args[2..],
                "--checking-account",
                DEFAULT_CHECKING_ACCOUNT,
            )?;
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

fn parse_db(args: &[String]) -> Result<ParsedArgs, CliError> {
    if parse_flag_present(args, "--help") || parse_flag_present(args, "-h") {
        return Ok(ParsedArgs {
            command: Command::Help(HelpTopic::Db),
        });
    }

    let subcommand = args.get(1).ok_or_else(|| CliError::MissingSubcommand {
        command: "db".to_owned(),
    })?;

    match subcommand.as_str() {
        "--help" | "-h" => Ok(ParsedArgs {
            command: Command::Help(HelpTopic::Db),
        }),
        "migrate" => Ok(ParsedArgs {
            command: Command::Db(DbCommand::Migrate),
        }),
        "status" => Ok(ParsedArgs {
            command: Command::Db(DbCommand::Status),
        }),
        _ => Err(CliError::UnknownSubcommand {
            command: "db".to_owned(),
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
        Some("db") => Ok(HelpTopic::Db),
        Some("import") => Ok(HelpTopic::Import),
        Some("fetch") => Ok(HelpTopic::Fetch),
        Some("reconcile") => Ok(HelpTopic::Reconcile),
        Some("month") => Ok(HelpTopic::Month),
        Some("close") => Ok(HelpTopic::Close),
        Some(subcommand) => Err(CliError::UnknownSubcommand {
            command: "help".to_owned(),
            subcommand: subcommand.to_owned(),
        }),
    }
}

fn parse_flag_value(args: &[String], flag: &str) -> Result<String, CliError> {
    let idx =
        args.iter()
            .position(|arg| arg == flag)
            .ok_or_else(|| CliError::MissingRequiredArg {
                flag: flag.to_owned(),
            })?;
    let value = args
        .get(idx + 1)
        .filter(|v| !v.starts_with("--"))
        .ok_or_else(|| CliError::MissingArgValue {
            flag: flag.to_owned(),
        })?;
    Ok(value.clone())
}

fn parse_optional_flag_value(args: &[String], flag: &str) -> Result<Option<String>, CliError> {
    let Some(idx) = args.iter().position(|arg| arg == flag) else {
        return Ok(None);
    };

    let value = args
        .get(idx + 1)
        .filter(|v| !v.starts_with("--"))
        .ok_or_else(|| CliError::MissingArgValue {
            flag: flag.to_owned(),
        })?;
    Ok(Some(value.clone()))
}

fn parse_optional_flag_value_with_default(
    args: &[String],
    flag: &str,
    default_value: &str,
) -> Result<String, CliError> {
    Ok(parse_optional_flag_value(args, flag)?.unwrap_or_else(|| default_value.to_owned()))
}

fn parse_flag_present(args: &[String], flag: &str) -> bool {
    args.iter().any(|arg| arg == flag)
}

fn parse_optional_parsed_flag<T: std::str::FromStr>(
    args: &[String],
    flag: &str,
    default_value: T,
) -> Result<T, CliError> {
    let Some(value) = parse_optional_flag_value(args, flag)? else {
        return Ok(default_value);
    };

    let Ok(parsed) = value.parse::<T>() else {
        return Err(CliError::InvalidArgValue {
            flag: flag.to_owned(),
            value,
        });
    };

    Ok(parsed)
}

fn parse_optional_parsed_value<T: std::str::FromStr>(
    args: &[String],
    flag: &str,
) -> Result<Option<T>, CliError> {
    let Some(value) = parse_optional_flag_value(args, flag)? else {
        return Ok(None);
    };

    let Ok(parsed) = value.parse::<T>() else {
        return Err(CliError::InvalidArgValue {
            flag: flag.to_owned(),
            value,
        });
    };

    Ok(Some(parsed))
}

fn parse_required_parsed_flag<T: std::str::FromStr>(
    args: &[String],
    flag: &str,
) -> Result<T, CliError> {
    let value = parse_flag_value(args, flag)?;
    let Ok(parsed) = value.parse::<T>() else {
        return Err(CliError::InvalidArgValue {
            flag: flag.to_owned(),
            value,
        });
    };

    Ok(parsed)
}

fn parse_paired_i64_flags(
    args: &[String],
    left_flag: &str,
    right_flag: &str,
) -> Result<(Option<i64>, Option<i64>), CliError> {
    let left = parse_optional_parsed_value::<i64>(args, left_flag)?;
    let right = parse_optional_parsed_value::<i64>(args, right_flag)?;

    match (left, right) {
        (Some(left), Some(right)) => Ok((Some(left), Some(right))),
        (None, None) => Ok((None, None)),
        _ => Err(CliError::MissingRequiredArg {
            flag: format!("{left_flag}' or '{right_flag}"),
        }),
    }
}

fn parse_amount_cents(args: &[String]) -> Result<i64, CliError> {
    parse_required_parsed_flag(args, "--amount-cents")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanCommand {
    Project {
        initial_net_worth_cents: i64,
        monthly_savings_cents: i64,
        months: u16,
    },
    Fire {
        monthly_expenses_cents: i64,
        safe_withdrawal_rate_pct: Option<u8>,
        liquid_assets_cents: Option<i64>,
        liabilities_cents: Option<i64>,
    },
}

fn execute_plan_command(command: &PlanCommand) -> Result<(), CliError> {
    match command {
        PlanCommand::Project {
            initial_net_worth_cents,
            monthly_savings_cents,
            months,
        } => crate::commands::plan::project(
            *initial_net_worth_cents,
            *monthly_savings_cents,
            *months,
        ),
        PlanCommand::Fire {
            monthly_expenses_cents,
            safe_withdrawal_rate_pct,
            liquid_assets_cents,
            liabilities_cents,
        } => crate::commands::plan::fire(
            *monthly_expenses_cents,
            *safe_withdrawal_rate_pct,
            *liquid_assets_cents,
            *liabilities_cents,
        ),
    }
}

fn parse_plan(args: &[String]) -> Result<ParsedArgs, CliError> {
    if parse_flag_present(args, "--help") || parse_flag_present(args, "-h") {
        return Ok(ParsedArgs {
            command: Command::Help(HelpTopic::Plan),
        });
    }

    let subcommand = args.get(1).ok_or_else(|| CliError::MissingSubcommand {
        command: "plan".to_owned(),
    })?;

    match subcommand.as_str() {
        "--help" | "-h" => Ok(ParsedArgs {
            command: Command::Help(HelpTopic::Plan),
        }),
        "project" => {
            let initial_net_worth_cents =
                parse_required_parsed_flag(&args[2..], "--initial-net-worth-cents")?;
            let monthly_savings_cents =
                parse_required_parsed_flag(&args[2..], "--monthly-savings-cents")?;
            let months = parse_required_parsed_flag(&args[2..], "--months")?;
            Ok(ParsedArgs {
                command: Command::Plan(PlanCommand::Project {
                    initial_net_worth_cents,
                    monthly_savings_cents,
                    months,
                }),
            })
        }
        "fire" => {
            let monthly_expenses_cents =
                parse_required_parsed_flag(&args[2..], "--monthly-expenses-cents")?;
            let safe_withdrawal_rate_pct =
                parse_optional_parsed_flag(&args[2..], "--safe-withdrawal-rate-pct", 4)?;
            let liquid_assets_cents =
                parse_optional_parsed_value(&args[2..], "--liquid-assets-cents")?;
            let liabilities_cents = parse_optional_parsed_value(&args[2..], "--liabilities-cents")?;
            Ok(ParsedArgs {
                command: Command::Plan(PlanCommand::Fire {
                    monthly_expenses_cents,
                    safe_withdrawal_rate_pct: Some(safe_withdrawal_rate_pct),
                    liquid_assets_cents,
                    liabilities_cents,
                }),
            })
        }
        _ => Err(CliError::UnknownSubcommand {
            command: "plan".to_owned(),
            subcommand: subcommand.clone(),
        }),
    }
}
