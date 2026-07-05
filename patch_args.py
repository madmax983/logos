import re

with open("crates/logos-cli/src/args.rs", "r") as f:
    content = f.read()

replacements = [
    (
        "    Report(ReportCommand),\n}",
        "    Report(ReportCommand),\n    Plan(PlanCommand),\n}"
    ),
    (
        "            Self::Help(HelpTopic::Report) => \"help.report\",\n",
        "            Self::Help(HelpTopic::Report) => \"help.report\",\n            Self::Help(HelpTopic::Plan) => \"help.plan\",\n"
    ),
    (
        "            Self::Report(ReportCommand::Month { .. }) => \"report.month\",\n",
        "            Self::Report(ReportCommand::Month { .. }) => \"report.month\",\n            Self::Plan(PlanCommand::Fire { .. }) => \"plan.fire\",\n"
    ),
    (
        "    Close,\n}",
        "    Close,\n    Plan,\n}"
    ),
    (
        "        Some(\"report\") => Ok(HelpTopic::Report),\n",
        "        Some(\"report\") => Ok(HelpTopic::Report),\n        Some(\"plan\") => Ok(HelpTopic::Plan),\n"
    ),
    (
        "        Command::Report(command) => execute_report_command(command),\n",
        "        Command::Report(command) => execute_report_command(command),\n        Command::Plan(command) => execute_plan_command(command),\n"
    ),
    (
        "        \"report\" => parse_report(&values),\n",
        "        \"report\" => parse_report(&values),\n        \"plan\" => parse_plan(&values),\n"
    )
]

for search, replace in replacements:
    if search not in content:
        raise ValueError(f"Anchor string not found:\n{search}")
    content = content.replace(search, replace)

plan_command_enum = """
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanCommand {
    Fire {
        monthly_expenses_cents: i64,
        liquid_assets_cents: i64,
        liabilities_cents: i64,
        upcoming_vests_cents: i64,
    },
}
"""
search_enum = "#[derive(Debug, Clone, PartialEq, Eq)]\npub enum ReportCommand {"
if search_enum not in content:
    raise ValueError(f"Anchor string not found:\n{search_enum}")
content = content.replace(search_enum, f"{plan_command_enum}\n#[derive(Debug, Clone, PartialEq, Eq)]\npub enum ReportCommand {{")

execute_plan = """
fn execute_plan_command(command: &PlanCommand) -> Result<(), CliError> {
    match command {
        PlanCommand::Fire {
            monthly_expenses_cents,
            liquid_assets_cents,
            liabilities_cents,
            upcoming_vests_cents,
        } => commands::plan::fire(
            *monthly_expenses_cents,
            *liquid_assets_cents,
            *liabilities_cents,
            *upcoming_vests_cents,
        ),
    }
}
"""
search_execute = "fn execute_report_command(command: &ReportCommand) -> Result<(), CliError> {"
if search_execute not in content:
    raise ValueError(f"Anchor string not found:\n{search_execute}")
content = content.replace(search_execute, f"{execute_plan}\nfn execute_report_command(command: &ReportCommand) -> Result<(), CliError> {{")

parse_plan = """
fn parse_plan_fire(args: &[String]) -> Result<ParsedArgs, CliError> {
    let monthly_expenses_cents = parse_required_parsed_flag(&args[2..], "--monthly-expenses-cents")?;
    let liquid_assets_cents = parse_optional_parsed_flag(&args[2..], "--liquid-assets-cents", 0)?;
    let liabilities_cents = parse_optional_parsed_flag(&args[2..], "--liabilities-cents", 0)?;
    let upcoming_vests_cents = parse_optional_parsed_flag(&args[2..], "--upcoming-vests-cents", 0)?;
    Ok(ParsedArgs {
        command: Command::Plan(PlanCommand::Fire {
            monthly_expenses_cents,
            liquid_assets_cents,
            liabilities_cents,
            upcoming_vests_cents,
        }),
    })
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
        "fire" => parse_plan_fire(args),
        _ => Err(CliError::UnknownSubcommand {
            command: "plan".to_owned(),
            subcommand: subcommand.clone(),
        }),
    }
}
"""
search_parse = "fn parse_report(args: &[String]) -> Result<ParsedArgs, CliError> {"
if search_parse not in content:
    raise ValueError(f"Anchor string not found:\n{search_parse}")
content = content.replace(search_parse, f"{parse_plan}\nfn parse_report(args: &[String]) -> Result<ParsedArgs, CliError> {{")

with open("crates/logos-cli/src/args.rs", "w") as f:
    f.write(content)
