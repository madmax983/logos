use crate::commands;
use core::fmt;

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
    pub command: Command,
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

    /// Executes the parsed command.
    ///
    /// # Errors
    ///
    /// Returns an error if the command cannot be executed.
    pub fn execute(&self) -> Result<(), CliError> {
        execute_command(&self.command)
    }
}

fn execute_command(command: &Command) -> Result<(), CliError> {
    match command {
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

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Help(HelpTopic),
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
