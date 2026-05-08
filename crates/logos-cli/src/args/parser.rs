use super::model::{
    AnalyticsCommand, BudgetCommand, CliError, CloseCommand, Command, DbCommand, FetchCommand,
    HelpTopic, ImportCommand, MonthCommand, ParsedArgs, ReconcileCommand, ReportCommand,
    TxnCommand,
};

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

const DEFAULT_BUDGET_CENTS: i64 = 0;
const DEFAULT_EXPENSE_ACCOUNT_PREFIX: &str = "expenses:";
const DEFAULT_CHECKING_ACCOUNT: &str = "assets:checking";
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
