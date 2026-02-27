use crate::args::{CliError, HelpTopic};

const GENERAL_HELP_TEXT: &str = "\
Usage: ledger <command> [options]

Commands:
  help [command]                    Show general or command help
  aletheia <subcommand>             Manage local AletheiaDB instance
  txn add ...                       Add a transaction
  budget set                        Set a budget value
  report month                      Show the current month report
";

const TXN_HELP_TEXT: &str = "\
Usage: ledger txn <subcommand> [options]

Subcommands:
  add --description <text> --debit-account <name> --credit-account <name> --amount-cents <i64>

Environment:
  LOGOS_DB_PATH                    Override embedded ledger store path
";

const BUDGET_HELP_TEXT: &str = "\
Usage: ledger budget <subcommand> [options]

Subcommands:
  set --month <YYYY-MM> --budget-cents <i64> --expense-account-prefix <prefix>
                                     Set budget target for month/scope (all optional)
";

const REPORT_HELP_TEXT: &str = "\
Usage: ledger report <subcommand> [options]

Subcommands:
  month --month <YYYY-MM> --checking-account <name>
                                     Show month-windowed report (both optional)
";

const ALETHEIA_HELP_TEXT: &str = "\
Usage: ledger aletheia <subcommand>

Subcommands:
  start                             Run local aletheia-server via cargo
  status                            Check /status health endpoint

Environment:
  ALETHEIADB_MANIFEST_PATH          Override AletheiaDB Cargo.toml location
  GALLIFREYDB_HOST                  Host used for status checks
  GALLIFREYDB_PORT                  Port used for status checks/server
";

/// Handles help output for general and command-specific help.
///
/// # Errors
///
/// This handler never errors.
pub fn show(topic: HelpTopic) -> Result<(), CliError> {
    let text = match topic {
        HelpTopic::General => GENERAL_HELP_TEXT,
        HelpTopic::Txn => TXN_HELP_TEXT,
        HelpTopic::Budget => BUDGET_HELP_TEXT,
        HelpTopic::Report => REPORT_HELP_TEXT,
        HelpTopic::Aletheia => ALETHEIA_HELP_TEXT,
    };
    println!("{text}");
    Ok(())
}
